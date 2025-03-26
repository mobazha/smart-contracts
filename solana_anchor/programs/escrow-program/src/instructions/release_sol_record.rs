use anchor_lang::prelude::*;
use crate::{state::*, error::*, utils::verify_signatures_with_timelock};

#[derive(Accounts)]
#[instruction(
    payment_amounts: Vec<u64>,
    signatures: Vec<Vec<u8>>
)]
pub struct ReleaseSolRecord<'info> {
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
            &[escrow_record.moderator.is_some() as u8],
            &escrow_record.unique_id
        ],
        bump = escrow_record.bump,
        constraint = escrow_record.status == TransactionStatus::Pending @ EscrowError::InvalidTransactionStatus
    )]
    pub escrow_record: Account<'info, SolEscrowRecord>,
    
    #[account(
        mut,
        seeds = [POOL_SEED, b"sol"],
        bump,
        constraint = sol_pool.is_active @ EscrowError::PoolIsInactive,
        constraint = sol_pool.total_sol_balance >= escrow_record.amount @ EscrowError::InsufficientPoolBalance
    )]
    pub sol_pool: Account<'info, FundPool>,
    
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
    
    /// CHECK: 接收方账户会在指令中验证
    #[account(mut)]
    pub recipient1: UncheckedAccount<'info>,
    
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
    ctx: Context<ReleaseSolRecord>,
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
    
    // 执行支付
    let recipients = vec![
        Some(ctx.accounts.recipient1.to_account_info()),
        ctx.accounts.recipient2.as_ref().map(|r| r.to_account_info()),
        ctx.accounts.recipient3.as_ref().map(|r| r.to_account_info()),
        ctx.accounts.recipient4.as_ref().map(|r| r.to_account_info()),
    ];
    
    // 内联 process_sol_payments 函数的逻辑
    let sol_pool_info = ctx.accounts.sol_pool.to_account_info();
    let system_program_info = ctx.accounts.system_program.to_account_info();
    let pool_address = ctx.accounts.sol_pool.key();
    let seeds = &[
        POOL_SEED,
        b"sol",
        &[ctx.accounts.sol_pool.bump],
    ];
    let signer_seeds = &[&seeds[..]];
    
    // 遍历所有支付目标
    for (i, amount) in payment_amounts.iter().enumerate() {
        if *amount == 0 {
            continue;
        }
        
        if let Some(recipient) = &recipients[i] {
            // 使用系统指令从资金池转账到接收者
            let ix = anchor_lang::solana_program::system_instruction::transfer(
                &pool_address,
                &recipient.key(),
                *amount,
            );
            
            anchor_lang::solana_program::program::invoke_signed(
                &ix,
                &[
                    sol_pool_info.clone(),
                    recipient.clone(),
                    system_program_info.clone(),
                ],
                signer_seeds,
            )?;
            
            // 添加转账日志
            msg!(
                "从资金池转账 {} lamports 到账户 {}", 
                amount, 
                recipient.key()
            );
        }
    }
    
    // 更新资金池余额
    let pool = &mut ctx.accounts.sol_pool;
    pool.total_sol_balance = pool.total_sol_balance.checked_sub(total_amount)
        .ok_or(EscrowError::InsufficientPoolBalance)?;
    
    // 更新交易记录状态
    let record = &mut ctx.accounts.escrow_record;
    record.status = TransactionStatus::Completed;
    record.completion_time = Some(ctx.accounts.clock.unix_timestamp);
    
    msg!(
        "SOL托管交易已完成: 买家={}, 卖家={}, 金额={}, ID={:?}", 
        record.buyer, 
        record.seller,
        record.amount,
        record.unique_id
    );
    
    Ok(())
} 