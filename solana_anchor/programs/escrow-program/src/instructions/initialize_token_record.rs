use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use anchor_spl::associated_token::AssociatedToken;
use crate::{state::*, error::*};

#[derive(Accounts)]
#[instruction(
    moderator: Option<Pubkey>,
    unique_id: [u8; 20],
    required_signatures: u8,
    unlock_hours: u64,
    amount: u64
)]
pub struct InitializeTokenRecord<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    
    /// CHECK: 卖家账户，由客户端指定
    pub seller: AccountInfo<'info>,
    
    pub token_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        seeds = [POOL_SEED, token_mint.key().as_ref()],
        bump,
        constraint = token_pool.is_active @ EscrowError::PoolIsInactive,
        constraint = token_pool.mint == token_mint.key() @ EscrowError::InvalidTokenAccount
    )]
    pub token_pool: Account<'info, TokenPool>,
    
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = token_pool,
        constraint = pool_token_account.key() == token_pool.token_account @ EscrowError::InvalidTokenAccount
    )]
    pub pool_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = buyer,
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init,
        payer = buyer,
        space = TokenEscrowRecord::LEN,
        seeds = [
            RECORD_SEED,
            buyer.key().as_ref(),
            seller.key().as_ref(),
            token_mint.key().as_ref(),
            &[moderator.is_some() as u8],
            &unique_id
        ],
        bump
    )]
    pub escrow_record: Account<'info, TokenEscrowRecord>,
    
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(
    ctx: Context<InitializeTokenRecord>,
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
    record.mint = ctx.accounts.token_mint.key();
    record.amount = amount;
    record.unlock_time = current_time + (unlock_hours as i64 * 3600);
    record.required_signatures = required_signatures;
    record.unique_id = unique_id;
    record.status = TransactionStatus::Pending;
    record.creation_time = current_time;
    record.completion_time = None;
    record.bump = ctx.bumps.escrow_record;

    // 获取代币池账户信息
    let pool = &mut ctx.accounts.token_pool;
    
    // 转移代币到资金池账户
    let transfer_to_pool_ix = anchor_spl::token::Transfer {
        from: ctx.accounts.buyer_token_account.to_account_info(),
        to: ctx.accounts.pool_token_account.to_account_info(),
        authority: ctx.accounts.buyer.to_account_info(),
    };
    
    let escrow_transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_to_pool_ix,
    );
    
    token::transfer(escrow_transfer_ctx, amount)?;
    
    // 更新资金池余额
    pool.total_balance = pool.total_balance.checked_add(amount)
        .ok_or(EscrowError::AmountOverflow)?;
    pool.total_transactions = pool.total_transactions.checked_add(1)
        .ok_or(EscrowError::AmountOverflow)?;

    msg!(
        "已创建代币托管记录: 买家={}, 卖家={}, 代币={}, 金额={}, ID={:?}",
        record.buyer,
        record.seller,
        record.mint,
        record.amount,
        record.unique_id
    );
    
    Ok(())
} 