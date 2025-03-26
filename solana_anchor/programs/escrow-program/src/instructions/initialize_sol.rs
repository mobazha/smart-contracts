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
pub struct InitializeSol<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    
    /// CHECK: 卖家账户，由客户端指定
    pub seller: AccountInfo<'info>,
    
    #[account(
        init,
        payer = buyer,
        space = SolEscrow::LEN,
        seeds = [
            b"sol_escrow",
            buyer.key().as_ref(),
            seller.key().as_ref(),
            &[moderator.is_some() as u8],  // 使用1字节表示是否有moderator
            &unique_id
        ],
        bump
    )]
    pub escrow_account: Account<'info, SolEscrow>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

pub fn handler(
    ctx: Context<InitializeSol>,
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
    
    // 获取escrow_account_info
    let escrow_account_info = ctx.accounts.escrow_account.to_account_info();
    
    // 初始化托管账户状态
    let escrow = &mut ctx.accounts.escrow_account;
    escrow.is_initialized = true;
    escrow.buyer = ctx.accounts.buyer.key();
    escrow.seller = ctx.accounts.seller.key();
    escrow.moderator = moderator;
    escrow.amount = amount;
    escrow.unlock_time = ctx.accounts.clock.unix_timestamp + (unlock_hours as i64 * 3600);
    escrow.required_signatures = required_signatures;
    escrow.unique_id = unique_id;
    escrow.bump = ctx.bumps.escrow_account;

    // 转移SOL到escrow账户
    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.buyer.key(),
        &escrow_account_info.key(),
        amount,
    );
    
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.buyer.to_account_info(),
            escrow_account_info,
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    msg!(
        "Initialized SOL escrow: Buyer={}, Seller={}, Amount={}, ID={:?}",
        escrow.buyer,
        escrow.seller,
        escrow.amount,
        escrow.unique_id
    );
    
    Ok(())
} 