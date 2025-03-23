package test

import (
	"context"
	"testing"

	"github.com/gagliardetto/solana-go"
	"github.com/gagliardetto/solana-go/programs/token"
	"github.com/gagliardetto/solana-go/rpc"
	"github.com/mobazha/solana2/client/escrow"
)

func TestNormalFlow(t *testing.T) {
	// 创建托管参数
	moderatorKey := moderator.PublicKey()
	params := escrow.CreateEscrowParams{
		Buyer:             buyer.PublicKey(),
		Seller:            seller.PublicKey(),
		Moderator:         &moderatorKey,
		UnlockHours:       1,
		RequiredSignature: 2,
	}

	// 获取托管地址
	escrowAddr, _, err := client.GetEscrowAddress(params)
	if err != nil {
		t.Fatal(err)
	}

	// 创建并存款
	amount := uint64(solana.LAMPORTS_PER_SOL / 10) // 0.1 SOL
	sig, err := client.CreateAndDeposit(params, amount, &buyer.PrivateKey)
	if err != nil {
		t.Fatal(err)
	}
	client.WaitForConfirmation(sig)

	// 验证初始状态
	state, err := client.GetEscrowState(escrowAddr)
	if err != nil {
		t.Fatal(err)
	}
	if state.Amount != amount {
		t.Errorf("金额不匹配: 期望 %d, 实际 %d", amount, state.Amount)
	}

	// 测试签名
	sig, err = client.Sign(escrowAddr, &buyer.PrivateKey)
	if err != nil {
		t.Fatal(err)
	}
	client.WaitForConfirmation(sig)

	sig, err = client.Sign(escrowAddr, &seller.PrivateKey)
	if err != nil {
		t.Fatal(err)
	}
	client.WaitForConfirmation(sig)

	// 释放资金
	paymentTargets := []escrow.PaymentTarget{
		{
			Recipient: seller.PublicKey(),
			Amount:    amount,
		},
	}

	sig, err = client.Release(escrowAddr, &seller.PrivateKey, paymentTargets)
	if err != nil {
		t.Fatal(err)
	}
	client.WaitForConfirmation(sig)

	// 验证最终状态
	state, err = client.GetEscrowState(escrowAddr)
	if err != nil {
		t.Fatal(err)
	}
	if state.State != escrow.EscrowStateCompleted {
		t.Errorf("状态不匹配: 期望 Completed, 实际 %v", state.State)
	}
}

func TestTimelock(t *testing.T) {
	// ... 实现时间锁测试
}

func TestMultipleDeposits(t *testing.T) {
	// ... 实现多次存款测试
}

func TestErrorCases(t *testing.T) {
	// ... 实现错误情况测试
}

func TestSplTokenFlow(t *testing.T) {
	// 创建代币 Mint
	mint := solana.NewWallet()

	// 创建 Mint 账户
	recent, err := client.RpcClient.GetRecentBlockhash(context.Background(), rpc.CommitmentFinalized)
	if err != nil {
		t.Fatal(err)
	}

	tx, err := solana.NewTransaction(
		[]solana.Instruction{
			token.NewInitializeMintInstruction(
				6,                 // decimals (uint8)
				mint.PublicKey(),  // mint account
				buyer.PublicKey(), // mint authority
				buyer.PublicKey(), // freeze authority (optional)
				token.ProgramID,   // program id
			).Build(),
		},
		recent.Value.Blockhash,
		solana.TransactionPayer(buyer.PublicKey()),
	)
	if err != nil {
		t.Fatal(err)
	}

	// 签名并发送创建 Mint 的交易
	_, err = tx.Sign(func(key solana.PublicKey) *solana.PrivateKey {
		if key.Equals(buyer.PublicKey()) {
			return &buyer.PrivateKey
		}
		return nil
	})
	if err != nil {
		t.Fatal(err)
	}

	sig1, err := client.RpcClient.SendTransaction(context.Background(), tx)
	if err != nil {
		t.Fatal(err)
	}
	client.WaitForConfirmation(sig1.String())

	// 创建买家的代币账户
	buyerATA, _, err := solana.FindAssociatedTokenAddress(
		buyer.PublicKey(),
		mint.PublicKey(),
	)
	if err != nil {
		t.Fatal(err)
	}

	// 创建托管参数
	moderatorKey := moderator.PublicKey()
	params := escrow.CreateEscrowParams{
		Buyer:             buyer.PublicKey(),
		Seller:            seller.PublicKey(),
		Moderator:         &moderatorKey,
		UnlockHours:       1,
		RequiredSignature: 2,
		TokenType: escrow.TokenType{
			IsSpl: true,
			Mint:  mint.PublicKey(),
		},
	}

	// 获取托管地址
	escrowAddr, _, err := client.GetEscrowAddress(params)
	if err != nil {
		t.Fatal(err)
	}

	// 创建托管的代币账户
	escrowATA, _, err := solana.FindAssociatedTokenAddress(
		escrowAddr,
		mint.PublicKey(),
	)
	if err != nil {
		t.Fatal(err)
	}

	// 创建并初始化托管
	sig, err := client.CreateAndDeposit(params, 1_000_000, &buyer.PrivateKey)
	if err != nil {
		t.Fatal(err)
	}
	client.WaitForConfirmation(sig)

	// 存入代币
	tx, err = solana.NewTransaction(
		[]solana.Instruction{
			token.NewTransferInstruction(
				1_000_000,
				buyerATA,
				escrowATA,
				buyer.PublicKey(),
				[]solana.PublicKey{},
			).Build(),
		},
		recent.Value.Blockhash,
		solana.TransactionPayer(buyer.PublicKey()),
	)

	sig1, err = client.RpcClient.SendTransaction(context.Background(), tx)
	if err != nil {
		t.Fatal(err)
	}
	client.WaitForConfirmation(sig1.String())

	// 签名并释放
	sig, err = client.Sign(escrowAddr, &buyer.PrivateKey)
	if err != nil {
		t.Fatal(err)
	}
	client.WaitForConfirmation(sig)

	sig, err = client.Sign(escrowAddr, &seller.PrivateKey)
	if err != nil {
		t.Fatal(err)
	}
	client.WaitForConfirmation(sig)

	// 创建卖家的代币账户
	sellerATA, _, err := solana.FindAssociatedTokenAddress(
		seller.PublicKey(),
		mint.PublicKey(),
	)
	if err != nil {
		t.Fatal(err)
	}

	// 释放代币
	paymentTargets := []escrow.PaymentTarget{
		{
			Recipient: sellerATA,
			Amount:    1_000_000,
		},
	}

	sig, err = client.ReleaseSpl(escrowAddr, escrowATA, &seller.PrivateKey, paymentTargets)
	if err != nil {
		t.Fatal(err)
	}
	client.WaitForConfirmation(sig)

	// 验证状态
	state, err := client.GetEscrowState(escrowAddr)
	if err != nil {
		t.Fatal(err)
	}
	if state.State != escrow.EscrowStateCompleted {
		t.Errorf("状态不匹配: 期望 Completed, 实际 %v", state.State)
	}
}
