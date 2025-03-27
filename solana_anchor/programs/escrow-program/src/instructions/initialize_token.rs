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
    pub associated_token_program: Program<'info, AssociatedToken>,
    
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = buyer,
    )]
    pub buyer_token_account: Account<'info, TokenAccount>,
    
    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = token_mint,
        associated_token::authority = escrow_account,
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
    require!(amount > 0, EscrowError::InvalidPaymentParameters);
    
    // 初始化托管账户状态
    let escrow = &mut ctx.accounts.escrow_account;
    
    // 使用 EscrowAccount::new 创建基础结构
    escrow.base = EscrowAccount::new(
        ctx.accounts.buyer.key(),
        ctx.accounts.seller.key(),
        moderator,
        required_signatures,
        ctx.accounts.clock.unix_timestamp + (unlock_hours as i64 * 3600),
        unique_id,
        amount,
        ctx.bumps.escrow_account,
    );
    
    // 验证签名要求
    escrow.base.validate_required_signatures()?;
    
    // 设置代币特有字段
    escrow.mint = ctx.accounts.token_mint.key();

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
    
    msg!(
        "Initialized token escrow: Buyer={}, Seller={}, Amount={}, ID={:?}",
        escrow.base.buyer,
        escrow.base.seller,
        escrow.base.amount,
        escrow.base.unique_id
    );
    
    Ok(())
} 