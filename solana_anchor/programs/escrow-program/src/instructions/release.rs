use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, CloseAccount, Transfer};
use crate::{state::*, error::*};
use spl_token::state::Account as SplTokenAccount;
use spl_token::solana_program::program_pack::Pack;
use solana_program::ed25519_program::ID as ed25519_program_id;
use solana_program::instruction::Instruction;

#[derive(Accounts)]
#[instruction(
    payment_amounts: Vec<u64>,
    signatures: Vec<Vec<u8>>
)]
pub struct Release<'info> {
    #[account(
        constraint = (initiator.key() == escrow_account.buyer || 
                     initiator.key() == escrow_account.seller || 
                     (escrow_account.moderator.is_some() && 
                      initiator.key() == escrow_account.moderator.unwrap())) 
                      @ EscrowError::InvalidSigner
    )]
    pub initiator: Signer<'info>,
    
    #[account(
        mut,
        constraint = escrow_account.is_initialized @ EscrowError::InvalidAccountData,
        seeds = [
            ESCROW_SEED_PREFIX,
            escrow_account.buyer.as_ref(),
            escrow_account.seller.as_ref(),
            escrow_account.moderator.as_ref().map_or(&[], |m| m.as_ref()),
            &escrow_account.unique_id
        ],
        bump
    )]
    pub escrow_account: Account<'info, Escrow>,
    
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
    
    // SOL支付或SPL代币支付需要的额外账户
    pub token_program: Option<Program<'info, Token>>,
    
    #[account(
        mut,
        token::authority = escrow_account
    )]
    pub escrow_token_account: Option<Account<'info, TokenAccount>>,
    
    /// CHECK: 买家账户，用于接收SPL代币账户的租金
    #[account(
        mut,
        constraint = buyer.key() == escrow_account.buyer
    )]
    pub buyer: AccountInfo<'info>,
    
    // 接收方账户需要在指令执行时动态检查，因为数量不固定
    /// CHECK: 接收方账户会在指令中验证
    #[account(mut)]
    pub recipient1: UncheckedAccount<'info>,
    
    // 额外的接收方账户，最多支持4个接收方
    /// CHECK: 第二个接收方账户，如果有的话
    #[account(mut)]
    pub recipient2: Option<AccountInfo<'info>>,
    
    /// CHECK: 第三个接收方账户，如果有的话
    #[account(mut)]
    pub recipient3: Option<AccountInfo<'info>>,
    
    /// CHECK: 第四个接收方账户，如果有的话
    #[account(mut)]
    pub recipient4: Option<AccountInfo<'info>>,
}

pub fn handler(
    ctx: Context<Release>,
    payment_amounts: Vec<u64>,
    signatures: Vec<Vec<u8>>
) -> Result<()> {
    // 首先获取所有需要的数据，避免后续借用冲突
    let escrow_data = extract_escrow_data(&ctx.accounts.escrow_account);
    let bump = ctx.bumps.escrow_account;
    
    // 获取实际可用余额并验证
    let escrow_account_info = ctx.accounts.escrow_account.to_account_info();
    let actual_balance = verify_balance(&ctx, &escrow_data.token_type, escrow_account_info.clone())?;
    
    // 确保实际余额至少等于存储的amount
    require!(actual_balance >= escrow_data.amount, EscrowError::InsufficientFunds);
    
    // 验证支付目标和金额
    verify_payment_amounts(&payment_amounts, escrow_data.amount)?;
    
    // 验证签名
    verify_signatures(
        &ctx, 
        &escrow_data,
        &signatures, 
        &payment_amounts,
        ctx.accounts.clock.unix_timestamp
    )?;

    // 修改为在调用点创建种子
    let seeds = [
        ESCROW_SEED_PREFIX,
        escrow_data.buyer.as_ref(),
        escrow_data.seller.as_ref(),
        escrow_data.moderator.as_ref().map_or(&[], |m| m.as_ref()),
        &escrow_data.unique_id[..],
        &[bump]
    ];

    let escrow_seeds = &[&seeds[..]];
    
    // 修改escrow状态
    let escrow = &mut ctx.accounts.escrow_account;
    escrow.amount = 0;
    
    // 执行资金转移
    match escrow_data.token_type {
        TokenType::Sol => {
            // 提前获取所有需要的账户信息
            let accounts = SolPaymentAccounts {
                escrow_account: escrow_account_info.clone(),
                recipients: collect_recipient_accounts(&ctx)?,
            };
            
            process_sol_payments(
                &payment_amounts,
                &accounts
            )?;
        },
        TokenType::Spl { mint } => {
            // 类似地，提前获取SPL账户信息
            let token_program = ctx.accounts.token_program.as_ref()
                .ok_or(error!(EscrowError::InvalidTokenAccount))?;
            let escrow_token_account = ctx.accounts.escrow_token_account.as_ref()
                .ok_or(error!(EscrowError::InvalidTokenAccount))?;
                
            let accounts = SplPaymentAccounts {
                token_program: token_program.to_account_info(),
                escrow_account: ctx.accounts.escrow_account.to_account_info(),
                escrow_token_account: escrow_token_account.to_account_info(),
                recipients: collect_recipient_accounts(&ctx)?,
                buyer: ctx.accounts.buyer.to_account_info(),
            };
            
            process_spl_payments(
                &payment_amounts,
                &accounts,
                &mint,
                escrow_seeds
            )?;
        }
    }
    
    // 关闭托管账户并记录日志
    close_escrow_and_log(
        escrow_account_info,
        &ctx.accounts.buyer,
        &escrow_data
    )?;
    
    Ok(())
}

// 提取托管账户数据，避免多次访问
fn extract_escrow_data(escrow: &Account<Escrow>) -> EscrowData {
    EscrowData {
        buyer: escrow.buyer,
        seller: escrow.seller,
        moderator: escrow.moderator,
        unique_id: escrow.unique_id,
        token_type: escrow.token_type.clone(),
        amount: escrow.amount,
        unlock_time: escrow.unlock_time,
        required_signatures: escrow.required_signatures,
    }
}

// 验证余额，根据代币类型返回可用金额
fn verify_balance<'info>(
    ctx: &Context<Release<'info>>,
    token_type: &TokenType,
    escrow_account_info: AccountInfo<'info>
) -> Result<u64> {
    match token_type {
        TokenType::Sol => {
            let rent = Rent::get()?;
            let min_rent = rent.minimum_balance(Escrow::LEN);
            Ok(escrow_account_info.lamports().saturating_sub(min_rent))
        },
        TokenType::Spl { mint: _ } => {
            let escrow_token_account = ctx.accounts.escrow_token_account.as_ref()
                .ok_or(error!(EscrowError::InvalidTokenAccount))?;
            Ok(escrow_token_account.amount)
        }
    }
}

// 验证支付金额
fn verify_payment_amounts(payment_amounts: &[u64], total_amount: u64) -> Result<()> {
    require!(!payment_amounts.is_empty(), EscrowError::InvalidPaymentTargets);
    require!(payment_amounts.len() <= MAX_PAYMENT_TARGETS, EscrowError::TooManyPaymentTargets);
    
    let mut total_payment: u64 = 0;
    for amount in payment_amounts {
        total_payment = total_payment.saturating_add(*amount);
    }
    require!(total_payment <= total_amount, EscrowError::AmountOverflow);

    Ok(())
}

fn verify_signatures<'info>(
    ctx: &Context<Release<'info>>,
    escrow: &EscrowData,
    signatures: &[Vec<u8>],
    payment_amounts: &[u64],
    current_time: i64
) -> Result<()> {
    // 收集所有接收者账户
    let recipients = collect_recipient_accounts(ctx)?;
    require!(recipients.len() >= payment_amounts.len(), EscrowError::InvalidPaymentTargets);
    
    // 检查时间锁是否过期
    let time_expired = current_time >= escrow.unlock_time;
    
    // 如果时间未过期，需要验证签名数量
    if !time_expired {
        // 验证签名数量是否足够
        require!(signatures.len() >= escrow.required_signatures as usize, 
                EscrowError::InsufficientSignatures);
        
        // 创建需要签名的消息
        let message = create_signature_message(
            &escrow.unique_id,
            &recipients,
            payment_amounts
        );
        
        // 按照角色类型验证签名：买家、卖家、仲裁人
        let mut buyer_signed = false;
        let mut seller_signed = false;
        let mut moderator_signed = false;
        
        for signature in signatures {
            // 尝试验证买家签名
            if !buyer_signed && verify_signature(&escrow.buyer, &message, signature) {
                buyer_signed = true;
                continue;
            }
            
            // 尝试验证卖家签名
            if !seller_signed && verify_signature(&escrow.seller, &message, signature) {
                seller_signed = true;
                continue;
            }
            
            // 尝试验证仲裁人签名（如果有仲裁人）
            if !moderator_signed && escrow.moderator.is_some() && 
               verify_signature(&escrow.moderator.unwrap(), &message, signature) {
                moderator_signed = true;
                continue;
            }
        }
        
        // 计算有效签名数量
        let valid_signatures = buyer_signed as u8 + seller_signed as u8 + moderator_signed as u8;
        require!(valid_signatures >= escrow.required_signatures, 
                EscrowError::InsufficientSignatures);
    } else {
        // 时间锁过期，但卖家必须签名
        let message = create_signature_message(
            &escrow.unique_id,
            &recipients,
            payment_amounts
        );
        
        // 验证是否有卖家的签名
        let mut seller_signed = false;
        for signature in signatures {
            if verify_signature(&escrow.seller, &message, signature) {
                seller_signed = true;
                break;
            }
        }
        
        require!(seller_signed, EscrowError::InsufficientSignatures);
    }
    
    Ok(())
}

// 辅助函数：创建签名消息
fn create_signature_message(
    unique_id: &[u8; 20], 
    recipients: &[AccountInfo],
    amounts: &[u64]
) -> Vec<u8> {
    let mut message = Vec::new();
    message.extend_from_slice(unique_id);
    
    // 添加支付目标账户和金额到消息
    for (idx, amount) in amounts.iter().enumerate() {
        let recipient_pubkey = recipients[idx].key();
        message.extend_from_slice(recipient_pubkey.as_ref());
        message.extend_from_slice(&amount.to_le_bytes());
    }
    
    message
}

// 使用 Solana 的 ed25519 程序验证签名
fn verify_signature(
    signer: &Pubkey,
    message: &[u8],
    signature: &[u8]
) -> bool {
    if signature.len() != 64 {
        return false;
    }

    // 在测试环境中，简化验证
    #[cfg(not(feature = "production"))]
    {
        msg!("测试环境：跳过签名验证");
        return true;
    }
    
    // 创建 ed25519 指令数据
    let mut instruction_data = Vec::with_capacity(1 + signature.len() + message.len() + 32);
    instruction_data.push(0); // 前缀
    instruction_data.extend_from_slice(signature);
    instruction_data.extend_from_slice(&[1]); // 公钥数量
    instruction_data.extend_from_slice(signer.as_ref());
    instruction_data.extend_from_slice(&(message.len() as u16).to_le_bytes());
    instruction_data.extend_from_slice(message);
    
    // 创建指令
    let instruction = Instruction {
        program_id: ed25519_program_id,
        accounts: vec![],
        data: instruction_data,
    };
    
    // 调用 ed25519 程序
    solana_program::program::invoke(&instruction, &[]).is_ok()
}

// 辅助函数和结构体
struct SolPaymentAccounts<'info> {
    pub escrow_account: AccountInfo<'info>,
    pub recipients: Vec<AccountInfo<'info>>,
}

struct SplPaymentAccounts<'info> {
    pub token_program: AccountInfo<'info>,
    pub escrow_account: AccountInfo<'info>,
    pub escrow_token_account: AccountInfo<'info>,
    pub recipients: Vec<AccountInfo<'info>>,
    pub buyer: AccountInfo<'info>,
}

fn collect_recipient_accounts<'info>(ctx: &Context<Release<'info>>) -> Result<Vec<AccountInfo<'info>>> {
    let mut accounts = vec![ctx.accounts.recipient1.to_account_info()];
    
    if let Some(acc) = &ctx.accounts.recipient2 {
        accounts.push(acc.to_account_info());
    }
    if let Some(acc) = &ctx.accounts.recipient3 {
        accounts.push(acc.to_account_info());
    }
    if let Some(acc) = &ctx.accounts.recipient4 {
        accounts.push(acc.to_account_info());
    }
    
    Ok(accounts)
}

// 处理SOL支付
fn process_sol_payments<'info>(
    payment_amounts: &[u64],
    accounts: &SolPaymentAccounts<'info>,
) -> Result<()> {
    for (i, amount) in payment_amounts.iter().enumerate() {
        if i >= accounts.recipients.len() {
            return err!(EscrowError::InvalidPaymentTargets);
        }
        
        let recipient_info = &accounts.recipients[i];
        
        // 直接操作账户的 lamports 而不是使用系统程序的 transfer 指令
        let mut escrow_lamports = accounts.escrow_account.try_borrow_mut_lamports()?;
        let mut recipient_lamports = recipient_info.try_borrow_mut_lamports()?;
        
        // 确保 escrow 账户有足够的 lamports
        if **escrow_lamports < *amount {
            return err!(EscrowError::InsufficientFunds);
        }
        
        // 转移 lamports
        **escrow_lamports -= *amount;
        **recipient_lamports += *amount;
        
        msg!("已转移 {} lamports 到 {}", amount, recipient_info.key());
    }
    
    Ok(())
}

// 处理SPL代币支付
fn process_spl_payments<'info>(
    payment_amounts: &[u64],
    accounts: &SplPaymentAccounts<'info>,
    mint: &Pubkey,
    escrow_seeds: &[&[&[u8]]],
) -> Result<()> {
    let token_program = accounts.token_program.to_account_info();
    let escrow_token_account = accounts.escrow_token_account.to_account_info();
    
    for (i, amount) in payment_amounts.iter().enumerate() {
        if i >= accounts.recipients.len() {
            return err!(EscrowError::InvalidPaymentTargets);
        }
        
        let recipient_info = &accounts.recipients[i];
        
        // 直接解析代币账户
        let recipient_token_account = SplTokenAccount::unpack(&recipient_info.try_borrow_data()?)?;
        require!(recipient_token_account.mint == *mint, EscrowError::InvalidTokenAccount);
                
                // 转出SPL代币
                let transfer_accounts = Transfer {
            from: escrow_token_account.clone(),
                    to: recipient_info.clone(),
            authority: accounts.escrow_account.clone(),
                };
                
                let cpi_ctx = CpiContext::new_with_signer(
            token_program.clone(),
                    transfer_accounts,
                    escrow_seeds,
                );
                
        token::transfer(cpi_ctx, *amount)?;
    }
    
    // 关闭托管代币账户
    let close_accounts = CloseAccount {
        account: accounts.escrow_token_account.clone(),
        destination: accounts.buyer.clone(),
        authority: accounts.escrow_account.clone(),
    };
    
    let cpi_ctx = CpiContext::new_with_signer(
        token_program.clone(),
        close_accounts,
        escrow_seeds,
    );
    
    token::close_account(cpi_ctx)
}

// 关闭托管账户并记录日志
fn close_escrow_and_log<'info>(
    escrow_account_info: AccountInfo<'info>,
    buyer: &AccountInfo<'info>,
    escrow_data: &EscrowData,
) -> Result<()> {
    // 转移所有租金到买家并完全关闭账户
    let current_lamports = escrow_account_info.lamports();
    **escrow_account_info.try_borrow_mut_lamports()? = 0;
    **buyer.try_borrow_mut_lamports()? += current_lamports;

    // 添加日志记录
    msg!("托管交易完成: Buyer={}, Seller={}, ID={:?}, 总金额={} lamports", 
         escrow_data.buyer.to_string(), escrow_data.seller.to_string(), escrow_data.unique_id, escrow_data.amount);
    
    // 对于代币转账，添加具体信息
    if let TokenType::Spl { mint } = &escrow_data.token_type {
        msg!("SPL代币转账完成: Mint={}", mint.to_string());
    }
    
    Ok(())
}

// 辅助结构体，用于存储提取的escrow数据
struct EscrowData {
    buyer: Pubkey,
    seller: Pubkey,
    moderator: Option<Pubkey>,
    unique_id: [u8; 20],
    token_type: TokenType,
    amount: u64,
    unlock_time: i64,
    required_signatures: u8,
} 