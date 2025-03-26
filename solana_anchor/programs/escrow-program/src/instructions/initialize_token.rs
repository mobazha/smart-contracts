use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use crate::{state::*, error::*};

#[derive(Accounts)]
#[instruction(
    moderator: Option<Pubkey>,
    unique_id: [u8; 20],
    required_signatures: u8,
    unlock_hours: u64,
    amount: u64
)]
pub struct InitializeToken<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    
    /// CHECK: 卖家账户，由客户端指定
    pub seller: AccountInfo<'info>,
    
    #[account(
        init,
        payer = buyer,
        space = TokenEscrow::LEN,
        seeds = [
            b"token_escrow",
            buyer.key().as_ref(),
            seller.key().as_ref(),
            &[moderator.is_some() as u8],  // 使用1字节表示是否有moderator
            &unique_id
        ],
        bump
    )]
    pub escrow_account: Account<'info, TokenEscrow>,
    
    pub token_program: Program<'info, Token>,
    pub token_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = buyer,
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = buyer,
        token::mint = token_mint,
        token::authority = escrow_account,
        seeds = [b"token_account", escrow_account.key().as_ref()],
        bump
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(
    ctx: Context<InitializeToken>,
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
    
    // 初始化托管账户状态
    let escrow = &mut ctx.accounts.escrow_account;
    escrow.is_initialized = true;
    escrow.buyer = ctx.accounts.buyer.key();
    escrow.seller = ctx.accounts.seller.key();
    escrow.moderator = moderator;
    escrow.mint = ctx.accounts.token_mint.key();  // 记录代币mint地址
    escrow.amount = amount;
    escrow.unlock_time = ctx.accounts.clock.unix_timestamp + (unlock_hours as i64 * 3600);
    escrow.required_signatures = required_signatures;
    escrow.unique_id = unique_id;
    escrow.bump = ctx.bumps.escrow_account;

    // 转移代币到escrow代币账户
    let transfer_to_escrow_ix = anchor_spl::token::Transfer {
        from: ctx.accounts.buyer_token_account.to_account_info(),
        to: ctx.accounts.escrow_token_account.to_account_info(),
        authority: ctx.accounts.buyer.to_account_info(),
    };
    
    let escrow_transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_to_escrow_ix,
    );
    
    token::transfer(escrow_transfer_ctx, amount)?;
    
    Ok(())
} 