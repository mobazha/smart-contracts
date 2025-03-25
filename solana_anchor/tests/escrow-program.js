import * as anchor from "@coral-xyz/anchor";
import { Keypair, PublicKey, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID, createMint, createAccount, mintTo, getAccount } from "@solana/spl-token";
import chai from "chai";
import { randomBytes, createHash } from "crypto";
import { keccak_256 } from '@noble/hashes/sha3';
import * as nobleSecp256k1 from '@noble/secp256k1';
import { hmac } from '@noble/hashes/hmac';
import { sha256 } from '@noble/hashes/sha256';
import nacl from 'tweetnacl';
import fs from 'fs';
import config from '../devnet-config.js';

const { assert, expect } = chai;
const BN = anchor.BN || (anchor.default && anchor.default.BN);

// 设置 HMAC-SHA256 实现
nobleSecp256k1.etc.hmacSha256Sync = (key, ...messages) => {
  const h = hmac.create(sha256, key);
  messages.forEach(msg => h.update(msg));
  return h.digest();
};

describe("escrow-program 测试", () => {
  // 配置测试环境
  const provider = new anchor.AnchorProvider(
    new anchor.web3.Connection(config.connection),
    anchor.Wallet.local(),
    { commitment: "confirmed" }
  );
  anchor.setProvider(provider);
  const program = anchor.workspace.EscrowProgram;
  
  // 根据环境创建测试账户
  let buyer, seller, moderator;
  let mainAccount;
  
  // 测试数据 - 使用配置中的金额
  const uniqueId = Buffer.from(randomBytes(20));
  const escrowAmount = config.testAmounts.escrow;
  const unlockHours = 24;
  const requiredSignatures = 2;
  
  // SPL代币测试数据
  let mintAuthority;
  let tokenMint;
  let buyerTokenAccount;
  let recipientTokenAccount;
  
  // 托管账户信息
  let escrowAccount;
  
  // 从主账户给各方充值
  const transferToAccount = async (toKeypair, amount) => {
    await provider.connection.sendTransaction(
      new anchor.web3.Transaction().add(
        anchor.web3.SystemProgram.transfer({
          fromPubkey: mainAccount.publicKey,
          toPubkey: toKeypair.publicKey,
          lamports: amount * LAMPORTS_PER_SOL,
        })
      ),
      [mainAccount]
    );
    console.log(`已向 ${toKeypair.publicKey.toString()} 转账 ${amount} SOL`);
  };
  
  // 初始化测试环境
  before(async () => {
    if (config.useRandomAccounts) {
      // 本地环境：使用随机生成的账户
      buyer = Keypair.generate();
      seller = Keypair.generate();
      moderator = Keypair.generate();
      console.log("使用随机生成的测试账户");
      
      // 为测试账户充值
      const airdropPromises = [buyer, seller, moderator].map(async (keypair) => {
        const signature = await provider.connection.requestAirdrop(
          keypair.publicKey, 
          10 * LAMPORTS_PER_SOL
        );
        return provider.connection.confirmTransaction(signature);
      });
      
      await Promise.all(airdropPromises);
      console.log("已为测试账户充值资金");
    } else {
      // Devnet 环境：使用预先充值的账户
      try {
        // 加载主账户
        mainAccount = anchor.web3.Keypair.fromSecretKey(
          Uint8Array.from(JSON.parse(fs.readFileSync(config.keypairPaths.main, 'utf-8')))
        );
        
          buyer = anchor.web3.Keypair.fromSecretKey(
            Uint8Array.from(JSON.parse(fs.readFileSync(config.keypairPaths.buyer, 'utf-8')))
          );
          seller = anchor.web3.Keypair.fromSecretKey(
            Uint8Array.from(JSON.parse(fs.readFileSync(config.keypairPaths.seller, 'utf-8')))
          );
          moderator = anchor.web3.Keypair.fromSecretKey(
            Uint8Array.from(JSON.parse(fs.readFileSync(config.keypairPaths.moderator, 'utf-8')))
          );
          console.log("使用预先充值的测试账户");
        
        // 打印各方地址
        console.log("主账户地址:", mainAccount.publicKey.toString());
        console.log("买家地址:", buyer.publicKey.toString());
        console.log("卖家地址:", seller.publicKey.toString());
        console.log("仲裁人地址:", moderator.publicKey.toString());
        
        // 执行转账
        await transferToAccount(buyer, 3);
        await transferToAccount(seller, 1);
        await transferToAccount(moderator, 1);
      } catch (e) {
        console.error("无法加载主账户:", e);
        console.log("回退到随机生成的账户");
        buyer = Keypair.generate();
        seller = Keypair.generate();
        moderator = Keypair.generate();
        
        // 为随机账户充值
        const airdropPromises = [buyer, seller, moderator].map(async (keypair) => {
          const signature = await provider.connection.requestAirdrop(
            keypair.publicKey, 
            10 * LAMPORTS_PER_SOL
          );
          return provider.connection.confirmTransaction(signature);
        });
        
        await Promise.all(airdropPromises);
      }
    }
    
    // 检查余额，确保有足够的资金
    const buyerBalance = await provider.connection.getBalance(buyer.publicKey);
    console.log(`买家余额: ${buyerBalance / LAMPORTS_PER_SOL} SOL`);
    
    // 如果余额不足，可以考虑跳过测试
    if (buyerBalance < 1 * LAMPORTS_PER_SOL) {
      console.warn("警告: 买家余额不足，测试可能会失败");
    }
    
    // 准备SPL代币测试环境
    mintAuthority = Keypair.generate();
    tokenMint = await createMint(
      provider.connection,
      buyer,
      mintAuthority.publicKey,
      null,
      9
    );
    
    buyerTokenAccount = await createAccount(
      provider.connection,
      buyer,
      tokenMint,
      buyer.publicKey
    );
    
    // 为买家铸造代币
    await mintTo(
      provider.connection,
      buyer,
      tokenMint,
      buyerTokenAccount,
      mintAuthority,
      10 * LAMPORTS_PER_SOL
    );
    
    // 创建卖家的代币账户用于测试
    recipientTokenAccount = await createAccount(
      provider.connection,
      seller,
      tokenMint,
      seller.publicKey
    );
    
    // 计算托管账户地址
    const [derivedEscrowAddress, derivedBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"),
        buyer.publicKey.toBuffer(),
        seller.publicKey.toBuffer(),
        moderator.publicKey.toBuffer(),
        uniqueId,
      ],
      program.programId
    );
    
    escrowAccount = derivedEscrowAddress;

    console.log("=== 调试信息 ===");
    console.log("buyer公钥:", buyer.publicKey.toString());
    console.log("seller公钥:", seller.publicKey.toString());
    console.log("moderator公钥:", moderator.publicKey.toString());
    console.log("tokenMint:", tokenMint.toString());
    console.log("buyerTokenAccount:", buyerTokenAccount.toString());
    console.log("=== 调试信息结束 ===");
  });
  
  it("初始化SOL托管账户", async () => {
    await program.methods
      .initialize(
        moderator.publicKey,
        Array.from(uniqueId),
        requiredSignatures,
        new BN(unlockHours),
        { sol: {} },
        new BN(escrowAmount)
      )
      .accounts({
        buyer: buyer.publicKey,
        seller: seller.publicKey,
        escrowAccount: escrowAccount,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        tokenProgram: null,
        tokenMint: null,
        buyerTokenAccount: null,
        escrowTokenAccount: null,
      })
      .signers([buyer])
      .rpc();
    
    // 验证托管账户状态
    const escrow = await program.account.escrow.fetch(escrowAccount);
    assert.equal(escrow.buyer.toString(), buyer.publicKey.toString());
    assert.equal(escrow.seller.toString(), seller.publicKey.toString());
    assert.equal(escrow.moderator?.toString(), moderator.publicKey.toString());
    assert.equal(escrow.amount.toString(), escrowAmount.toString());
    assert.equal(escrow.requiredSignatures, requiredSignatures);
    assert.isTrue(escrow.isInitialized);
    
    // 验证资金是否已转移到托管账户
    const escrowBalance = await provider.connection.getBalance(escrowAccount);
    // 获取实际的租金豁免金额
    const rentExemption = await provider.connection.getMinimumBalanceForRentExemption(
      program.account.escrow.size
    );
    // 考虑交易费用和其他开销
    const expectedBalance = escrowAmount + rentExemption;
    
    // 使用approximately而不是isAtLeast，允许有小额误差
    assert.approximately(
      escrowBalance,
      expectedBalance,
      10000  // 增加到10000 lamports
    );
  });
  
  it("初始化SPL代币托管账户", async () => {
    const tokenUniqueId = Buffer.from(randomBytes(20));
    const tokenAmount = 0.2 * LAMPORTS_PER_SOL;
    
    // 计算新的托管账户地址
    const [tokenEscrowAccount, tokenEscrowBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"),
        buyer.publicKey.toBuffer(),
        seller.publicKey.toBuffer(),
        moderator.publicKey.toBuffer(),
        tokenUniqueId,
      ],
      program.programId
    );
    
    // 查找托管代币账户地址
    const [escrowTokenAccount] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("token_account"),
        tokenEscrowAccount.toBuffer(),
      ],
      program.programId
    );
    
    await program.methods
      .initialize(
        moderator.publicKey,
        Array.from(tokenUniqueId),
        requiredSignatures,
        new BN(unlockHours),
          { spl: { mint: tokenMint } },
        new BN(tokenAmount)
      )
      .accounts({
        buyer: buyer.publicKey,
        seller: seller.publicKey,
        escrowAccount: tokenEscrowAccount,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        tokenMint: tokenMint,
        buyerTokenAccount: buyerTokenAccount,
        escrowTokenAccount: escrowTokenAccount,
      })
      .signers([buyer])
      .rpc();
    
    // 验证托管账户状态
    const escrow = await program.account.escrow.fetch(tokenEscrowAccount);
    assert.equal(escrow.amount.toString(), tokenAmount.toString());
    
    // 验证代币已转移到托管账户
    const tokenAccountInfo = await getAccount(provider.connection, escrowTokenAccount);
    assert.equal(tokenAccountInfo.amount.toString(), tokenAmount.toString());
  });
  
  it("使用单个签名释放SOL", async () => {
    // 创建简单托管（只需要1个签名）
    const simpleUniqueId = Buffer.from(randomBytes(20));
    const simpleAmount = 0.05 * LAMPORTS_PER_SOL;
    
    // 计算简单托管地址
    const [simpleEscrowAccount, simpleEscrowBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"),
        buyer.publicKey.toBuffer(),
        seller.publicKey.toBuffer(),
        Buffer.from([]), // 没有仲裁人
        simpleUniqueId,
      ],
      program.programId
    );
    
    // 初始化简单托管
    await program.methods
      .initialize(
        null, // 没有仲裁人
        Array.from(simpleUniqueId),
        1, // 只需要1个签名
        new BN(unlockHours),
        { sol: {} },
        new BN(simpleAmount)
      )
      .accounts({
        buyer: buyer.publicKey,
        seller: seller.publicKey,
        escrowAccount: simpleEscrowAccount,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        tokenProgram: null,
        tokenMint: null,
        buyerTokenAccount: null,
        escrowTokenAccount: null,
      })
      .signers([buyer])
      .rpc();
    
    // 创建消息进行签名
    const paymentAmount = simpleAmount;
    let message = Buffer.from([...simpleUniqueId]);
    message = Buffer.concat([message, seller.publicKey.toBuffer()]);
    message = Buffer.concat([message, Buffer.from(new BN(paymentAmount).toArray('le', 8))]);

    // 计算消息哈希 - 这必须与程序代码匹配
    const messageHash = keccak_256(message);

    // 使用 Solana 密钥对签名
    const signature = nacl.sign.detached(
      message,
      buyer.secretKey
    );

    // 打印调试信息
    console.log("消息:", Buffer.from(message).toString('hex'));
    console.log("消息哈希:", Buffer.from(messageHash).toString('hex'));
    console.log("签名:", Buffer.from(signature).toString('hex'));
    
    // 获取卖家初始余额
    const sellerBalanceBefore = await provider.connection.getBalance(seller.publicKey);
    
    // 释放资金
    await program.methods
      .release(
        [new BN(paymentAmount)],
        [Buffer.from(signature)]
      )
      .accounts({
        initiator: buyer.publicKey,
        escrowAccount: simpleEscrowAccount,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        systemProgram: SystemProgram.programId,
        tokenProgram: null,
        escrowTokenAccount: null,
        buyer: buyer.publicKey,
        recipient1: seller.publicKey,
        recipient2: null,
        recipient3: null,
        recipient4: null,
      })
      .signers([buyer])
      .rpc();
    
    // 验证卖家收到了资金
    const sellerBalanceAfter = await provider.connection.getBalance(seller.publicKey);
    assert.approximately(
      sellerBalanceAfter - sellerBalanceBefore,
      paymentAmount,
      0.01 * LAMPORTS_PER_SOL
    );
    
    // 验证托管账户已关闭
    const escrowAccountInfo = await provider.connection.getAccountInfo(simpleEscrowAccount);
    assert.isNull(escrowAccountInfo);
  });
  
  it("使用多个签名释放SOL", async () => {
    // 创建新的多签名托管
    const multiUniqueId = Buffer.from(randomBytes(20));
    const multiAmount = 0.08 * LAMPORTS_PER_SOL;
    
    // 计算多签名托管地址
    const [multiEscrowAccount, multiEscrowBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"),
        buyer.publicKey.toBuffer(),
        seller.publicKey.toBuffer(),
        moderator.publicKey.toBuffer(),
        multiUniqueId,
      ],
      program.programId
    );
    
    // 初始化多签名托管
    await program.methods
      .initialize(
        moderator.publicKey,
        Array.from(multiUniqueId),
        2, // 需要2个签名
        new BN(unlockHours),
        { sol: {} },
        new BN(multiAmount)
      )
      .accounts({
        buyer: buyer.publicKey,
        seller: seller.publicKey,
        escrowAccount: multiEscrowAccount,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        tokenProgram: null,
        tokenMint: null,
        buyerTokenAccount: null,
        escrowTokenAccount: null,
      })
      .signers([buyer])
      .rpc();
    
    // 创建消息进行签名
    const paymentAmount = multiAmount;
    let message = Buffer.from([...multiUniqueId]);
    message = Buffer.concat([message, seller.publicKey.toBuffer()]);
    message = Buffer.concat([message, Buffer.from(new BN(paymentAmount).toArray('le', 8))]);
    
    // 获取买家和卖家的签名
    const buyerSignature = nacl.sign.detached(message, buyer.secretKey);
    const sellerSignature = nacl.sign.detached(message, seller.secretKey);
    
    // 获取卖家初始余额
    const sellerBalanceBefore = await provider.connection.getBalance(seller.publicKey);
    
    // 释放资金 - 使用两个签名
    await program.methods
      .release(
        [new BN(paymentAmount)],
        [Buffer.from(buyerSignature), Buffer.from(sellerSignature)] // 提供两个签名
      )
      .accounts({
        initiator: buyer.publicKey,
        escrowAccount: multiEscrowAccount,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        systemProgram: SystemProgram.programId,
        tokenProgram: null,
        escrowTokenAccount: null,
        buyer: buyer.publicKey,
        recipient1: seller.publicKey,
        recipient2: null,
        recipient3: null,
        recipient4: null,
      })
      .signers([buyer])
      .rpc();
    
    // 验证卖家收到了资金
    const sellerBalanceAfter = await provider.connection.getBalance(seller.publicKey);
    assert.approximately(
      sellerBalanceAfter - sellerBalanceBefore,
      paymentAmount,
      0.01 * LAMPORTS_PER_SOL
    );
  });
  
  it("释放SPL代币", async () => {
    // 创建新的SPL代币托管
    const tokenUniqueId = Buffer.from(randomBytes(20));
    const tokenAmount = 0.2 * LAMPORTS_PER_SOL;
    
    // 计算SPL托管地址
    const [tokenEscrowAccount, tokenEscrowBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"),
        buyer.publicKey.toBuffer(),
        seller.publicKey.toBuffer(),
        moderator.publicKey.toBuffer(),
        tokenUniqueId,
      ],
      program.programId
    );
    
    // 查找托管代币账户地址
    const [escrowTokenAccount] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("token_account"),
        tokenEscrowAccount.toBuffer(),
      ],
      program.programId
    );
    
    // 初始化托管，设置为只需要1个签名
    await program.methods
      .initialize(
        moderator.publicKey,
        Array.from(tokenUniqueId),
        1, // 只需要1个签名
        new BN(unlockHours),
        { spl: { mint: tokenMint } },
        new BN(tokenAmount)
      )
      .accounts({
        buyer: buyer.publicKey,
        seller: seller.publicKey,
        escrowAccount: tokenEscrowAccount,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        tokenProgram: TOKEN_PROGRAM_ID,
        tokenMint: tokenMint,
        buyerTokenAccount: buyerTokenAccount,
        escrowTokenAccount: escrowTokenAccount,
      })
      .signers([buyer])
      .rpc();
    
    // 创建消息进行签名
    let message = Buffer.from([...tokenUniqueId]);
    message = Buffer.concat([message, recipientTokenAccount.toBuffer()]);
    message = Buffer.concat([message, Buffer.from(new BN(tokenAmount).toArray('le', 8))]);
    
    // 使用买家的密钥签名
    const buyerSignature = nacl.sign.detached(message, buyer.secretKey);
    
    // 获取释放前的余额
    const balanceBefore = (await getAccount(provider.connection, recipientTokenAccount)).amount;
    
    // 释放代币
    await program.methods
      .release(
        [new BN(tokenAmount)],
        [Buffer.from(buyerSignature)]
      )
      .accounts({
        initiator: buyer.publicKey,
        escrowAccount: tokenEscrowAccount,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        escrowTokenAccount: escrowTokenAccount,
        buyer: buyer.publicKey,
        recipient1: recipientTokenAccount,
        recipient2: null,
        recipient3: null,
        recipient4: null,
      })
      .signers([buyer])
      .rpc();
    
    // 验证代币已转移
    const balanceAfter = (await getAccount(provider.connection, recipientTokenAccount)).amount;
    assert.equal(
      balanceAfter - balanceBefore,
      BigInt(tokenAmount)
    );
  });
  
  it("时间锁过期后只需卖家签名", async () => {
    // 创建时间锁过期的托管
    const expiredUniqueId = Buffer.from(randomBytes(20));
    const expiredAmount = 0.3 * LAMPORTS_PER_SOL;
    
    // 计算地址
    const [expiredEscrowAccount, expiredEscrowBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"),
        buyer.publicKey.toBuffer(),
        seller.publicKey.toBuffer(),
        moderator.publicKey.toBuffer(),
        expiredUniqueId,
      ],
      program.programId
    );
    
    // 初始化托管（0小时超时，立即过期）
    await program.methods
      .initialize(
        moderator.publicKey,
        Array.from(expiredUniqueId),
        2, // 2个签名
        new BN(0), // 立即过期
        { sol: {} },
        new BN(expiredAmount)
      )
      .accounts({
        buyer: buyer.publicKey,
        seller: seller.publicKey,
        escrowAccount: expiredEscrowAccount,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        tokenProgram: null,
        tokenMint: null,
        buyerTokenAccount: null,
        escrowTokenAccount: null,
      })
      .signers([buyer])
      .rpc();
    
    // 创建消息进行签名
    let message = Buffer.from([...expiredUniqueId]);
    message = Buffer.concat([message, seller.publicKey.toBuffer()]);
    message = Buffer.concat([message, Buffer.from(new BN(expiredAmount).toArray('le', 8))]);
    
    // 只使用卖家签名
    const sellerSignature = nacl.sign.detached(
      message,
      seller.secretKey
    );
    
    // 获取释放前的余额
    const sellerBalanceBefore = await provider.connection.getBalance(seller.publicKey);
    
    // 释放资金
    await program.methods
      .release(
        [new BN(expiredAmount)],
        [Buffer.from(sellerSignature)]
      )
      .accounts({
        initiator: seller.publicKey,
        escrowAccount: expiredEscrowAccount,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        systemProgram: SystemProgram.programId,
        tokenProgram: null,
        escrowTokenAccount: null,
        buyer: buyer.publicKey,
        recipient1: seller.publicKey,
        recipient2: null,
        recipient3: null,
        recipient4: null,
      })
      .signers([seller])
      .rpc();
    
    // 验证卖家收到了资金
    const sellerBalanceAfter = await provider.connection.getBalance(seller.publicKey);
    assert.approximately(
      sellerBalanceAfter - sellerBalanceBefore,
      expiredAmount - 0.00001 * LAMPORTS_PER_SOL, // 减去交易费用
      0.01 * LAMPORTS_PER_SOL
    );
  });
  
  it("测试分割支付到多个接收者", async () => {
    // 创建新的托管
    const splitUniqueId = Buffer.from(randomBytes(20));
    const splitAmount = 1 * LAMPORTS_PER_SOL;
    
    // 接收方
    const recipient1 = seller.publicKey;
    const recipient2 = moderator.publicKey;
    
    // 计算地址
    const [splitEscrowAccount, splitEscrowBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("escrow"),
        buyer.publicKey.toBuffer(),
        seller.publicKey.toBuffer(),
        moderator.publicKey.toBuffer(),
        splitUniqueId,
      ],
      program.programId
    );
    
    // 初始化托管
    await program.methods
      .initialize(
        moderator.publicKey,
        Array.from(splitUniqueId),
        1, // 1个签名
        new BN(unlockHours),
        { sol: {} },
        new BN(splitAmount)
      )
      .accounts({
        buyer: buyer.publicKey,
        seller: seller.publicKey,
        escrowAccount: splitEscrowAccount,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        tokenProgram: null,
        tokenMint: null,
        buyerTokenAccount: null,
        escrowTokenAccount: null,
      })
      .signers([buyer])
      .rpc();
    
    // 分割金额
    const amount1 = 0.7 * LAMPORTS_PER_SOL;
    const amount2 = 0.3 * LAMPORTS_PER_SOL;
    
    // 创建消息进行签名
    let message = Buffer.from([...splitUniqueId]);
    message = Buffer.concat([message, recipient1.toBuffer()]);
    message = Buffer.concat([message, Buffer.from(new BN(amount1).toArray('le', 8))]);
    message = Buffer.concat([message, recipient2.toBuffer()]);
    message = Buffer.concat([message, Buffer.from(new BN(amount2).toArray('le', 8))]);
    
    // 使用 Solana 密钥对签名
    const signature = nacl.sign.detached(
      message,
      buyer.secretKey
    );
    
    // 获取释放前的余额
    const balance1Before = await provider.connection.getBalance(recipient1);
    const balance2Before = await provider.connection.getBalance(recipient2);
    
    // 释放分割的资金
    await program.methods
      .release(
        [new BN(amount1), new BN(amount2)],
        [Buffer.from(signature)]
      )
      .accounts({
        initiator: buyer.publicKey,
        escrowAccount: splitEscrowAccount,
        clock: anchor.web3.SYSVAR_CLOCK_PUBKEY,
        systemProgram: SystemProgram.programId,
        tokenProgram: null,
        escrowTokenAccount: null,
        buyer: buyer.publicKey,
        recipient1: recipient1,
        recipient2: recipient2,
        recipient3: null,
        recipient4: null,
      })
      .signers([buyer])
      .rpc();
    
    // 验证两个接收者都收到了资金
    const balance1After = await provider.connection.getBalance(recipient1);
    const balance2After = await provider.connection.getBalance(recipient2);
    
    assert.approximately(balance1After - balance1Before, amount1, 0.01 * LAMPORTS_PER_SOL);
    assert.approximately(balance2After - balance2Before, amount2, 0.01 * LAMPORTS_PER_SOL);
});

after(async () => {
  try {
    // 将剩余资金转回主账户
    const mainAccount = new PublicKey("97mEtQYthR5c3hbeut4CYb5pU9FGZTop8k7aXraBanaV");
    const rentExempt = await provider.connection.getMinimumBalanceForRentExemption(0);
    
    // 转移账户剩余资金的函数
    const transferRemainingFunds = async (fromKeypair, accountType) => {
      const balance = await provider.connection.getBalance(fromKeypair.publicKey);
      if (balance > rentExempt + 10000) {
        const transferAmount = balance - rentExempt - 10000; // 保留一些用于交易费
        
        await provider.connection.sendTransaction(
          new anchor.web3.Transaction().add(
            anchor.web3.SystemProgram.transfer({
              fromPubkey: fromKeypair.publicKey,
              toPubkey: mainAccount,
              lamports: transferAmount,
            })
          ),
          [fromKeypair]
        );
        
        console.log(`已将 ${transferAmount / LAMPORTS_PER_SOL} SOL 从${accountType}账户转回主账户`);
      }
    };
    
    // 转移所有测试账户的剩余资金
    await Promise.all([
      transferRemainingFunds(buyer, "买家"),
      transferRemainingFunds(seller, "卖家"),
      transferRemainingFunds(moderator, "仲裁人")
    ]);
    
  } catch (e) {
    console.error("清理资金时出错:", e);
  }
  });
}); 