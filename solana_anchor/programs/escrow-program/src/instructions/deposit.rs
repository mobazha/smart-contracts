use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::{state::*, error::*};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,
    
    #[account(
        mut,
        constraint = escrow_account.state == EscrowState::Active @ EscrowError::AlreadyCompleted,
        constraint = escrow_account.is_initialized @ EscrowError::InvalidAccountData
    )]
    pub escrow_account: Account<'info, Escrow>,
    
    pub system_program: Program<'info, System>,
    
    // SOL存款不需要额外账户，SPL代币存款需要以下账户
    pub token_program: Option<Program<'info, Token>>,
    
    #[account(
        mut,
        token::authority = depositor
    )]
    pub source: Option<Account<'info, TokenAccount>>,
    
    #[account(
        mut,
        token::authority = escrow_account
    )]
    pub destination: Option<Account<'info, TokenAccount>>,
}

pub fn handler(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    require!(amount > 0, EscrowError::ZeroAmount);
    
    let escrow = &mut ctx.accounts.escrow_account;
    
    match escrow.token_type {
        TokenType::Sol => {
            // 转移SOL
            let cpi_context = CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.depositor.to_account_info(),
                    to: ctx.accounts.escrow_account.to_account_info(),
                },
            );
            
            anchor_lang::system_program::transfer(cpi_context, amount)?;
        },
        TokenType::Spl(_) => {
            // 验证代币账户存在
            let source = ctx.accounts.source.as_ref()
                .ok_or(error!(EscrowError::InvalidTokenAccount))?;
            let destination = ctx.accounts.destination.as_ref()
                .ok_or(error!(EscrowError::InvalidTokenAccount))?;
            let token_program = ctx.accounts.token_program.as_ref()
                .ok_or(error!(EscrowError::InvalidTokenAccount))?;
            
            // 转移SPL代币
            let cpi_accounts = Transfer {
                from: source.to_account_info(),
                to: destination.to_account_info(),
                authority: ctx.accounts.depositor.to_account_info(),
            };
            
            let cpi_context = CpiContext::new(
                token_program.to_account_info(),
                cpi_accounts,
            );
            
            token::transfer(cpi_context, amount)?;
        }
    }
    
    // 更新托管账户金额
    escrow.amount = escrow.amount.checked_add(amount)
        .ok_or(error!(EscrowError::AmountOverflow))?;
    
    msg!("已存入金额: {}", amount);
    
    Ok(())
} 