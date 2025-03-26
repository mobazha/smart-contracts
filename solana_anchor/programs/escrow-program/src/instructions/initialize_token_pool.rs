use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::*;

#[derive(Accounts)]
pub struct InitializeTokenPool<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub token_mint: Account<'info, Mint>,
    
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = token_mint,
        associated_token::authority = pool_account,
    )]
    pub pool_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init,
        payer = authority,
        space = TokenPool::LEN,
        seeds = [POOL_SEED, token_mint.key().as_ref()],
        bump
    )]
    pub pool_account: Account<'info, TokenPool>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<InitializeTokenPool>) -> Result<()> {
    let pool = &mut ctx.accounts.pool_account;
    
    pool.authority = ctx.accounts.authority.key();
    pool.mint = ctx.accounts.token_mint.key();
    pool.token_account = ctx.accounts.pool_token_account.key();
    pool.total_balance = 0;
    pool.total_transactions = 0;
    pool.is_active = true;
    pool.bump = ctx.bumps.pool_account;
    
    msg!(
        "代币资金池已初始化: 管理者={}, 代币铸造={}", 
        pool.authority,
        pool.mint
    );
    
    Ok(())
} 