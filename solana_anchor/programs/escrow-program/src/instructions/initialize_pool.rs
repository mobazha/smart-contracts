use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct InitializeSolPool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = FundPool::LEN,
        seeds = [POOL_SEED, b"sol"],
        bump
    )]
    pub pool_account: Account<'info, FundPool>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<InitializeSolPool>) -> Result<()> {
    let pool = &mut ctx.accounts.pool_account;
    
    pool.authority = ctx.accounts.authority.key();
    pool.total_sol_balance = 0;
    pool.total_transactions = 0;
    pool.is_active = true;
    pool.bump = ctx.bumps.pool_account;
    
    msg!("SOL资金池已初始化: 管理者={}", pool.authority);
    
    Ok(())
} 