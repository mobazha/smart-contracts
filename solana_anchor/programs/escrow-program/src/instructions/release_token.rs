use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, CloseAccount};
use crate::{state::*, error::*, utils::{verify_payment_amounts, verify_signatures_with_timelock, close_escrow_and_return_rent}};

#[derive(Accounts)]
#[instruction(
    payment_amounts: Vec<u64>,
    signatures: Vec<Vec<u8>>
)]
pub struct ReleaseToken<'info> {
    #[account(
        constraint = (initiator.key() == escrow_account.buyer || 
                     initiator.key() == escrow_account.seller || 
                     (escrow_account.moderator.is_some() && 
                      initiator.key() == escrow_account.moderator.unwrap())) 
                      @ EscrowError::InvalidSigner
    )]
    pub initiator: Signer<'info>,
    
    #[account(
        mut,
        constraint = escrow_account.is_initialized @ EscrowError::InvalidAccountData,
        seeds = [
            b"token_escrow",
            escrow_account.buyer.as_ref(),
            escrow_account.seller.as_ref(),
            &[escrow_account.moderator.is_some() as u8],
            &escrow_account.unique_id
        ],
        bump = escrow_account.bump
    )]
    pub escrow_account: Account<'info, TokenEscrow>,
    
    pub token_program: Program<'info, Token>,
    
    #[account(
        mut,
        token::authority = escrow_account
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: 买家账户，用于接收SPL代币账户的租金
    #[account(mut)]
    pub buyer: AccountInfo<'info>,
    
    /// CHECK: 接收方代币账户会在指令中验证
    #[account(mut)]
    pub recipient1: Account<'info, TokenAccount>,
    
    /// CHECK: 第二个接收方代币账户，如果有的话
    #[account(mut)]
    pub recipient2: Option<Account<'info, TokenAccount>>,
    
    /// CHECK: 第三个接收方代币账户，如果有的话
    #[account(mut)]
    pub recipient3: Option<Account<'info, TokenAccount>>,
    
    /// CHECK: 第四个接收方代币账户，如果有的话
    #[account(mut)]
    pub recipient4: Option<Account<'info, TokenAccount>>,
    
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
}

pub fn handler(
    ctx: Context<ReleaseToken>,
    payment_amounts: Vec<u64>,
    signatures: Vec<Vec<u8>>
) -> Result<()> {
    // 验证支付金额
    verify_payment_amounts(&payment_amounts, &ctx.accounts.escrow_account)?;
    
    // 验证签名（包含时间锁检查）
    verify_signatures_with_timelock(
        &ctx.accounts.escrow_account,
        &signatures,
        &payment_amounts,
        ctx.accounts.clock.unix_timestamp,
    )?;
    
    let seeds = &[
        b"token_escrow",
        ctx.accounts.escrow_account.buyer.as_ref(),
        ctx.accounts.escrow_account.seller.as_ref(),
        &[ctx.accounts.escrow_account.moderator.is_some() as u8],
        &ctx.accounts.escrow_account.unique_id,
        &[ctx.accounts.escrow_account.bump],
    ];
    let signer_seeds = &[&seeds[..]];

    // 直接处理每个支付
    let recipients = [
        Some((&ctx.accounts.recipient1, payment_amounts.get(0))),
        ctx.accounts.recipient2.as_ref().map(|r| (r, payment_amounts.get(1))),
        ctx.accounts.recipient3.as_ref().map(|r| (r, payment_amounts.get(2))),
        ctx.accounts.recipient4.as_ref().map(|r| (r, payment_amounts.get(3))),
    ];

    process_token_payments(
        &ctx.accounts.escrow_token_account,
        &ctx.accounts.escrow_account,
        &ctx.accounts.token_program,
        &recipients,
        signer_seeds,
    )?;
    
    // 关闭托管代币账户
    let close_accounts = CloseAccount {
        account: ctx.accounts.escrow_token_account.to_account_info(),
        destination: ctx.accounts.buyer.to_account_info(),
        authority: ctx.accounts.escrow_account.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        close_accounts,
        signer_seeds
    );
    
    token::close_account(cpi_ctx)?;
    
    // 关闭托管账户并返回租金
    close_escrow_and_return_rent(
        &ctx.accounts.escrow_account.to_account_info(),
        &ctx.accounts.buyer,
    )?;
    
    Ok(())
}

fn process_token_payments<'info>(
    escrow_token_account: &Account<'info, TokenAccount>,
    escrow_account: &Account<'info, TokenEscrow>,
    token_program: &Program<'info, Token>,
    recipients: &[Option<(&Account<'info, TokenAccount>, Option<&u64>)>],
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    for recipient in recipients.iter().flatten() {
        let (recipient_account, amount) = recipient;
        if let Some(&amount) = amount {
            let recipient_token = recipient_account.to_account_info();
            require!(
                recipient_account.mint == escrow_account.mint,
                EscrowError::InvalidTokenAccount
            );
            
            // 添加转账日志
            msg!(
                "Transfer {} tokens to account {}, Mint: {}", 
                amount, 
                recipient_token.key(), 
                escrow_account.mint
            );
            
            let transfer_ix = anchor_spl::token::Transfer {
                from: escrow_token_account.to_account_info(),
                to: recipient_token,
                authority: escrow_account.to_account_info(),
            };
            
            let cpi_ctx = CpiContext::new_with_signer(
                token_program.to_account_info(),
                transfer_ix,
                signer_seeds,
            );
            token::transfer(cpi_ctx, amount)?;
        }
    }
    
    // 添加完成日志
    msg!(
        "Token escrow completed: Buyer={}, Seller={}, ID={:?}", 
        escrow_account.buyer, 
        escrow_account.seller,
        escrow_account.unique_id
    );
    
    Ok(())
} 