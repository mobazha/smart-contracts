use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use crate::{state::*, error::*, utils::{verify_signatures_with_timelock}};

#[derive(Accounts)]
#[instruction(
    payment_amounts: Vec<u64>,
    signatures: Vec<Vec<u8>>
)]
pub struct ReleaseTokenRecord<'info> {
    #[account(
        constraint = (initiator.key() == escrow_record.buyer || 
                     initiator.key() == escrow_record.seller || 
                     (escrow_record.moderator.is_some() && 
                      initiator.key() == escrow_record.moderator.unwrap())) 
                      @ EscrowError::InvalidSigner
    )]
    pub initiator: Signer<'info>,
    
    #[account(
        mut,
        seeds = [
            RECORD_SEED,
            escrow_record.buyer.as_ref(),
            escrow_record.seller.as_ref(),
            escrow_record.mint.as_ref(),
            &[escrow_record.moderator.is_some() as u8],
            &escrow_record.unique_id
        ],
        bump = escrow_record.bump,
        constraint = escrow_record.status == TransactionStatus::Pending @ EscrowError::InvalidTransactionStatus
    )]
    pub escrow_record: Account<'info, TokenEscrowRecord>,
    
    #[account(
        mut,
        seeds = [POOL_SEED, escrow_record.mint.as_ref()],
        bump,
        constraint = token_pool.is_active @ EscrowError::PoolIsInactive,
        constraint = token_pool.mint == escrow_record.mint @ EscrowError::InvalidTokenAccount,
        constraint = token_pool.total_balance >= escrow_record.amount @ EscrowError::InsufficientPoolBalance
    )]
    pub token_pool: Account<'info, TokenPool>,
    
    #[account(
        mut,
        token::mint = escrow_record.mint,
        token::authority = token_pool,
        constraint = pool_token_account.key() == token_pool.token_account @ EscrowError::InvalidTokenAccount
    )]
    pub pool_token_account: Account<'info, TokenAccount>,
    
    pub token_program: Program<'info, Token>,
    
    /// CHECK: 接收方代币账户会在指令中验证
    #[account(mut)]
    pub recipient1: Account<'info, TokenAccount>,
    
    /// CHECK: 第二个接收方代币账户，如果有的话
    #[account(mut)]
    pub recipient2: Option<Account<'info, TokenAccount>>,
    
    /// CHECK: 第三个接收方代币账户，如果有的话
    #[account(mut)]
    pub recipient3: Option<Account<'info, TokenAccount>>,
    
    /// CHECK: 第四个接收方代币账户，如果有的话
    #[account(mut)]
    pub recipient4: Option<Account<'info, TokenAccount>>,
    
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<ReleaseTokenRecord>,
    payment_amounts: Vec<u64>,
    signatures: Vec<Vec<u8>>
) -> Result<()> {
    // 验证支付金额
    let total_amount: u64 = payment_amounts.iter().sum();
    require!(total_amount <= ctx.accounts.escrow_record.amount, EscrowError::PaymentAmountMismatch);
    
    // 验证签名（包含时间锁检查）
    verify_signatures_with_timelock(
        &*ctx.accounts.escrow_record,
        &signatures,
        &payment_amounts,
        ctx.accounts.clock.unix_timestamp,
    )?;
    
    let seeds = &[
        POOL_SEED,
        ctx.accounts.escrow_record.mint.as_ref(),
        &[ctx.accounts.token_pool.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    let recipients = [
        (Some(&ctx.accounts.recipient1), payment_amounts.get(0)),
        (ctx.accounts.recipient2.as_ref(), payment_amounts.get(1)),
        (ctx.accounts.recipient3.as_ref(), payment_amounts.get(2)),
        (ctx.accounts.recipient4.as_ref(), payment_amounts.get(3)),
    ];

    process_token_payments(
        &ctx.accounts.pool_token_account,
        &ctx.accounts.token_pool,
        &ctx.accounts.token_program,
        &recipients,
        signer_seeds,
    )?;
    
    // 更新资金池余额
    let pool = &mut ctx.accounts.token_pool;
    pool.total_balance = pool.total_balance.checked_sub(total_amount)
        .ok_or(EscrowError::InsufficientPoolBalance)?;
    
    // 更新交易记录状态
    let record = &mut ctx.accounts.escrow_record;
    record.status = TransactionStatus::Completed;
    record.completion_time = Some(ctx.accounts.clock.unix_timestamp);
    
    msg!(
        "代币托管交易已完成: 买家={}, 卖家={}, 代币={}, 金额={}, ID={:?}", 
        record.buyer, 
        record.seller,
        record.mint,
        record.amount,
        record.unique_id
    );
    
    Ok(())
}

fn process_token_payments<'info>(
    pool_token_account: &Account<'info, TokenAccount>,
    token_pool: &Account<'info, TokenPool>,
    token_program: &Program<'info, Token>,
    recipients: &[(Option<&Account<'info, TokenAccount>>, Option<&u64>)],
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    for (recipient_opt, amount_opt) in recipients {
        if let (Some(recipient), Some(&amount)) = (recipient_opt, amount_opt) {
            if amount == 0 {
                continue;
            }
            
            let recipient_token = recipient.to_account_info();
            require!(
                recipient.mint == token_pool.mint,
                EscrowError::InvalidTokenAccount
            );
            
            // 添加转账日志
            msg!(
                "从资金池转账 {} 代币到账户 {}, 代币铸造: {}", 
                amount, 
                recipient_token.key(), 
                token_pool.mint
            );
            
            let transfer_ix = anchor_spl::token::Transfer {
                from: pool_token_account.to_account_info(),
                to: recipient_token,
                authority: token_pool.to_account_info(),
            };
            
            let cpi_ctx = CpiContext::new_with_signer(
                token_program.to_account_info(),
                transfer_ix,
                signer_seeds,
            );
            token::transfer(cpi_ctx, amount)?;
        }
    }
    
    Ok(())
} 