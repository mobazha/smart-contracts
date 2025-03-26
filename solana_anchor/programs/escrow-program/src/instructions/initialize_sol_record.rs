use anchor_lang::prelude::*;
use crate::{state::*, error::*};

#[derive(Accounts)]
#[instruction(
    moderator: Option<Pubkey>,
    unique_id: [u8; 20],
    required_signatures: u8,
    unlock_hours: u64,
    amount: u64
)]
pub struct InitializeSolRecord<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    
    /// CHECK: 卖家账户，由客户端指定
    pub seller: AccountInfo<'info>,
    
    #[account(
        mut,
        seeds = [POOL_SEED, b"sol"],
        bump,
        constraint = sol_pool.is_active @ EscrowError::PoolIsInactive
    )]
    pub sol_pool: Account<'info, FundPool>,
    
    #[account(
        init,
        payer = buyer,
        space = SolEscrowRecord::LEN,
        seeds = [
            RECORD_SEED,
            buyer.key().as_ref(),
            seller.key().as_ref(),
            &[moderator.is_some() as u8],
            &unique_id
        ],
        bump
    )]
    pub escrow_record: Account<'info, SolEscrowRecord>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(
    ctx: Context<InitializeSolRecord>,
    moderator: Option<Pubkey>,
    unique_id: [u8; 20],
    required_signatures: u8,
    unlock_hours: u64,
    amount: u64,
) -> Result<()> {
    // 验证参数
    require!(required_signatures > 0, EscrowError::NoSignature);
    require!(required_signatures <= MAX_REQUIRED_SIGNATURES, EscrowError::TooManyRequiredSignatures);
    require!(amount > 0, EscrowError::ZeroAmount);
    
    // 获取当前时间戳
    let current_time = ctx.accounts.clock.unix_timestamp;
    
    // 初始化交易记录账户
    let record = &mut ctx.accounts.escrow_record;
    record.buyer = ctx.accounts.buyer.key();
    record.seller = ctx.accounts.seller.key();
    record.moderator = moderator;
    record.amount = amount;
    record.unlock_time = current_time + (unlock_hours as i64 * 3600);
    record.required_signatures = required_signatures;
    record.unique_id = unique_id;
    record.status = TransactionStatus::Pending;
    record.creation_time = current_time;
    record.completion_time = None;
    record.bump = ctx.bumps.escrow_record;

    // 获取资金池账户信息
    let pool = &mut ctx.accounts.sol_pool;
    let sol_pool_info = pool.to_account_info();
    
    // 转移SOL到资金池账户
    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.buyer.key(),
        &pool.key(),
        amount,
    );
    
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.buyer.to_account_info(),
            sol_pool_info,
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;
    
    // 更新资金池余额
    pool.total_sol_balance = pool.total_sol_balance.checked_add(amount)
        .ok_or(EscrowError::AmountOverflow)?;
    pool.total_transactions = pool.total_transactions.checked_add(1)
        .ok_or(EscrowError::AmountOverflow)?;

    msg!(
        "已创建SOL托管记录: 买家={}, 卖家={}, 金额={}, ID={:?}",
        record.buyer,
        record.seller,
        record.amount,
        record.unique_id
    );
    
    Ok(())
} 