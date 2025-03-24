use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::{state::*, error::*};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub depositor: Signer<'info>,
    
    #[account(
        mut,
        constraint = escrow_account.is_initialized @ EscrowError::InvalidAccountData,
        constraint = escrow_account.state == EscrowState::Active @ EscrowError::AlreadyCompleted,
        constraint = (depositor.key() == escrow_account.buyer || 
                     depositor.key() == escrow_account.seller || 
                     (escrow_account.moderator.is_some() && 
                      depositor.key() == escrow_account.moderator.unwrap())) 
                      @ EscrowError::InvalidSigner,
        seeds = [
            ESCROW_SEED_PREFIX,
            escrow_account.buyer.as_ref(),
            escrow_account.seller.as_ref(),
            escrow_account.moderator.as_ref().map_or(&[], |m| m.as_ref()),
            &escrow_account.unique_id
        ],
        bump
    )]
    pub escrow_account: Account<'info, Escrow>,
    
    pub system_program: Program<'info, System>,
    
    // SPL代币相关账户（可选）
    pub token_program: Option<Program<'info, Token>>,
    
    #[account(mut)]
    pub source: Option<Account<'info, TokenAccount>>,
    
    #[account(
        mut,
        token::authority = escrow_account
    )]
    pub destination: Option<Account<'info, TokenAccount>>,
}

pub fn handler(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    require!(amount > 0, EscrowError::ZeroAmount);
    
    // 验证代币账户的mint
    if let TokenType::Spl(mint) = ctx.accounts.escrow_account.token_type {
        // 验证source
        if let Some(source) = &ctx.accounts.source {
            require!(source.mint == mint, EscrowError::InvalidTokenAccount);
        } else {
            return err!(EscrowError::InvalidTokenAccount);
        }
        
        // 验证destination
        if let Some(dest) = &ctx.accounts.destination {
            require!(dest.mint == mint, EscrowError::InvalidTokenAccount);
        } else {
            return err!(EscrowError::InvalidTokenAccount);
        }
    }
    
    // 获取escrow账户的信息克隆，避免后续借用冲突
    let escrow_account_info = ctx.accounts.escrow_account.to_account_info();
    
    let escrow = &mut ctx.accounts.escrow_account;
    
    match escrow.token_type {
        TokenType::Sol => {
            // 转移SOL
            let cpi_context = CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: ctx.accounts.depositor.to_account_info(),
                    to: escrow_account_info,  // 使用预先克隆的账户信息
                },
            );
            
            anchor_lang::system_program::transfer(cpi_context, amount)?;
        },
        TokenType::Spl(_) => {
            // 转移SPL代币
            let cpi_accounts = Transfer {
                from: ctx.accounts.source.as_ref().unwrap().to_account_info(),
                to: ctx.accounts.destination.as_ref().unwrap().to_account_info(),
                authority: ctx.accounts.depositor.to_account_info(),
            };
            
            let cpi_context = CpiContext::new(
                ctx.accounts.token_program.as_ref().unwrap().to_account_info(),
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