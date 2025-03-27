use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::{state::*, error::*, utils::{close_escrow_and_return_rent, process_release}};

#[derive(Accounts)]
#[instruction(
    payment_amounts: Vec<u64>,
    signatures: Vec<Vec<u8>>
)]
pub struct ReleaseToken<'info> {
    #[account(
        constraint = (initiator.key() == escrow_account.base.buyer || 
                     initiator.key() == escrow_account.base.seller || 
                     (escrow_account.base.moderator.is_some() && 
                      initiator.key() == escrow_account.base.moderator.unwrap())) 
                      @ EscrowError::Unauthorized
    )]
    pub initiator: Signer<'info>,
    
    #[account(
        mut,
        constraint = escrow_account.base.is_initialized @ EscrowError::ValidationFailed,
        seeds = [
            b"token_escrow",
            escrow_account.base.buyer.as_ref(),
            escrow_account.base.seller.as_ref(),
            &[escrow_account.base.moderator.is_some() as u8],
            &escrow_account.base.unique_id
        ],
        bump = escrow_account.base.bump
    )]
    pub escrow_account: Account<'info, TokenEscrow>,
    
    #[account(
        mut,
        constraint = escrow_token_account.mint == escrow_account.mint @ EscrowError::ValidationFailed,
        constraint = escrow_token_account.owner == escrow_account.key() @ EscrowError::ValidationFailed,
    )]
    pub escrow_token_account: Account<'info, TokenAccount>,
    
    pub clock: Sysvar<'info, Clock>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    
    /// CHECK: 买家账户
    #[account(mut, address = escrow_account.base.buyer @ EscrowError::ValidationFailed)]
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
    
    /// Sysvar Instructions account
    #[account(address = solana_program::sysvar::instructions::ID)]
    pub sysvar_instructions: AccountInfo<'info>,
}

pub fn handler(
    ctx: Context<ReleaseToken>,
    payment_amounts: Vec<u64>,
    signatures: Vec<Vec<u8>>
) -> Result<()> {
    // 收集接收方代币账户和公钥
    let recipient_accounts = [
        Some(&ctx.accounts.recipient1),
        ctx.accounts.recipient2.as_ref(),
        ctx.accounts.recipient3.as_ref(),
        ctx.accounts.recipient4.as_ref(),
    ];
    
    let recipient_pubkeys = recipient_accounts.iter()
        .map(|acc| acc.map(|a| a.owner))
        .collect::<Vec<Option<Pubkey>>>();
    
    // 使用统一的处理函数
    process_release(
        &*ctx.accounts.escrow_account,
        &signatures,
        &payment_amounts,
        &recipient_pubkeys[..],
        ctx.accounts.clock.unix_timestamp,
        &ctx.accounts.sysvar_instructions,
        || {
            // 执行代币转账逻辑
            transfer_tokens_to_recipients(
                &ctx,
                &payment_amounts,
                &recipient_accounts
            )?;
            
            // 关闭托管账户并返回租金
            close_escrow_and_return_rent(
                &ctx.accounts.escrow_account.to_account_info(),
                &ctx.accounts.buyer,
            )
        },
    )
}

// 代币转账逻辑
fn transfer_tokens_to_recipients<'info>(
    ctx: &Context<ReleaseToken<'info>>,
    amounts: &[u64],
    recipients: &[Option<&Account<'info, TokenAccount>>],
) -> Result<()> {
    // 验证所有接收方账户的代币类型
    for recipient in recipients.iter().flatten() {
        require!(
            recipient.mint == ctx.accounts.escrow_account.mint,
            EscrowError::ValidationFailed
        );
    }
    
    // 获取PDA签名种子
    let escrow_seed = &[
        b"token_escrow",
        ctx.accounts.escrow_account.base.buyer.as_ref(),
        ctx.accounts.escrow_account.base.seller.as_ref(),
        &[ctx.accounts.escrow_account.base.moderator.is_some() as u8],
        &ctx.accounts.escrow_account.base.unique_id,
        &[ctx.accounts.escrow_account.base.bump],
    ];
    
    // 转账代币给各接收方
    for (i, amount) in amounts.iter().enumerate() {
        if let Some(recipient) = recipients[i] {
            // 添加转账日志
            msg!(
                "Transfer {} tokens to account {}", 
                amount, 
                recipient.key()
            );
            
            // 使用PDA签名执行转账
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.escrow_token_account.to_account_info(),
                        to: recipient.to_account_info(),
                        authority: ctx.accounts.escrow_account.to_account_info(),
                    },
                    &[escrow_seed],
                ),
                *amount,
            )?;
        }
    }
    
    // 添加完成日志
    msg!(
        "Token escrow completed: Buyer={}, Seller={}, ID={:?}", 
        ctx.accounts.escrow_account.base.buyer, 
        ctx.accounts.escrow_account.base.seller,
        ctx.accounts.escrow_account.base.unique_id
    );
    
    Ok(())
} 