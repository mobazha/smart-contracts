use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};
use crate::{state::*, error::*};

#[derive(Accounts)]
#[instruction(
    moderator: Option<Pubkey>,
    unique_id: [u8; 20],
    required_signatures: u8,
    unlock_hours: u64,
    token_type: TokenType,
    amount: u64
)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    
    /// CHECK: 卖家账户，由客户端指定
    pub seller: AccountInfo<'info>,
    
    #[account(
        init,
        payer = buyer,
        space = Escrow::LEN,
        seeds = [
            ESCROW_SEED_PREFIX,
            buyer.key().as_ref(),
            seller.key().as_ref(),
            moderator.as_ref().map_or(&[], |m| m.as_ref()),
            &unique_id
        ],
        bump
    )]
    pub escrow_account: Account<'info, Escrow>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
    
    // 以下账户仅SPL代币支付时需要
    pub token_program: Option<Program<'info, Token>>,
    pub token_mint: Option<Account<'info, Mint>>,
    
    #[account(
        init_if_needed,
        payer = buyer,
        token::mint = token_mint,
        token::authority = buyer,
    )]
    pub buyer_token_account: Option<Account<'info, TokenAccount>>,
    
    #[account(
        init_if_needed,
        payer = buyer,
        token::mint = token_mint,
        token::authority = escrow_account
    )]
    pub escrow_token_account: Option<Account<'info, TokenAccount>>,
}

pub fn handler(
    ctx: Context<Initialize>,
    moderator: Option<Pubkey>,
    unique_id: [u8; 20],
    required_signatures: u8,
    unlock_hours: u64,
    token_type: TokenType,
    amount: u64
) -> Result<()> {
    // 验证参数
    require!(required_signatures > 0, EscrowError::NoSignature);
    require!(required_signatures <= MAX_REQUIRED_SIGNATURES, EscrowError::TooManyRequiredSignatures);
    require!(amount > 0, EscrowError::ZeroAmount);
    
    // 初始化托管账户状态
    let escrow = &mut ctx.accounts.escrow_account;
    escrow.buyer = ctx.accounts.buyer.key();
    escrow.seller = ctx.accounts.seller.key();
    escrow.moderator = moderator;
    escrow.token_type = token_type.clone();
    escrow.amount = amount;
    escrow.unlock_time = ctx.accounts.clock.unix_timestamp + (unlock_hours as i64 * 3600);
    escrow.required_signatures = required_signatures;
    escrow.is_initialized = true;
    escrow.unique_id = unique_id;

    // 处理资金转移
    match token_type {
        TokenType::Sol => {
            // 转移SOL到escrow账户
            let ix = anchor_lang::solana_program::system_instruction::transfer(
                &ctx.accounts.buyer.key(),
                &ctx.accounts.escrow_account.key(),
                amount,
            );
            
            anchor_lang::solana_program::program::invoke(
                &ix,
                &[
                    ctx.accounts.buyer.to_account_info(),
                    ctx.accounts.escrow_account.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            )?;
        },
        TokenType::Spl(mint) => {
            // 验证代币相关账户
            let token_program = ctx.accounts.token_program.as_ref()
                .ok_or(error!(EscrowError::InvalidTokenAccount))?;
            let token_mint = ctx.accounts.token_mint.as_ref()
                .ok_or(error!(EscrowError::InvalidTokenAccount))?;
            let buyer_token_account = ctx.accounts.buyer_token_account.as_ref()
                .ok_or(error!(EscrowError::InvalidTokenAccount))?;
            let escrow_token_account = ctx.accounts.escrow_token_account.as_ref()
                .ok_or(error!(EscrowError::InvalidTokenAccount))?;
            
            // 验证代币mint地址
            require!(token_mint.key() == mint, EscrowError::InvalidTokenAccount);
            
            // 转移代币到escrow代币账户
            let transfer_to_escrow_ix = anchor_spl::token::Transfer {
                from: buyer_token_account.to_account_info(),
                to: escrow_token_account.to_account_info(),
                authority: ctx.accounts.buyer.to_account_info(),
            };
            
            let escrow_transfer_ctx = CpiContext::new(
                token_program.to_account_info(),
                transfer_to_escrow_ix,
            );
            
            token::transfer(escrow_transfer_ctx, amount)?;
        }
    }
    
    Ok(())
} 