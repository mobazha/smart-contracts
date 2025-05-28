use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};
use anchor_spl::associated_token::AssociatedToken;
use crate::{state::*, error::*, utils::{close_escrow_and_return_rent, bytes_to_hex_string, construct_message, verify_ed25519_instructions, verify_payment_amounts, verify_signatures_without_timelock}};

#[derive(Accounts)]
#[instruction(
    payment_amounts: Vec<u64>,
    signatures: Vec<Vec<u8>>
)]
pub struct ReleaseToken<'info> {
    // 验证发起者是否是买家、卖家或moderator（不用这个约束）
    // #[account(
    //     constraint = (initiator.key() == escrow_account.base.buyer || 
    //                  initiator.key() == escrow_account.base.seller || 
    //                  (escrow_account.base.moderator.is_some() && 
    //                   initiator.key() == escrow_account.base.moderator.unwrap())) 
    //                   @ EscrowError::Unauthorized
    // )]
    #[account(mut)]
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
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    /// CHECK: Sysvar Instructions account
    #[account(address = solana_program::sysvar::instructions::ID)]
    pub sysvar_instructions: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    
    pub token_mint: Account<'info, Mint>,
    
    /// CHECK: 买家账户
    #[account(mut, address = escrow_account.base.buyer @ EscrowError::ValidationFailed)]
    pub buyer: AccountInfo<'info>,
    
    /// CHECK: 第一个接收方owner
    pub recipient1: AccountInfo<'info>,
    /// CHECK: 第一个接收方ATA
    #[account(
        init_if_needed,
        payer = initiator,
        associated_token::mint = token_mint,
        associated_token::authority = recipient1,
    )]
    pub recipient1_ata: Account<'info, TokenAccount>,
    
    /// CHECK: 第二个接收方owner
    pub recipient2: Option<AccountInfo<'info>>,
    /// CHECK: 第二个接收方ATA
    #[account(
        init_if_needed,
        payer = initiator,
        associated_token::mint = token_mint,
        associated_token::authority = recipient2,
    )]
    pub recipient2_ata: Option<Account<'info, TokenAccount>>,
    
    /// CHECK: 第三个接收方owner
    pub recipient3: Option<AccountInfo<'info>>,
    /// CHECK: 第三个接收方ATA
    #[account(
        init_if_needed,
        payer = initiator,
        associated_token::mint = token_mint,
        associated_token::authority = recipient3,
    )]
    pub recipient3_ata: Option<Account<'info, TokenAccount>>,
}

pub fn handler(
    ctx: Context<ReleaseToken>,
    payment_amounts: Vec<u64>,
    signatures: Vec<Vec<u8>>
) -> Result<()> {
    // 验证接收方数量
    require!(
        payment_amounts.len() >= 1 && payment_amounts.len() <= 3,
        EscrowError::InvalidRecipientCount
    );
    
    let recipient_accounts = [
        Some(&ctx.accounts.recipient1_ata),
        ctx.accounts.recipient2_ata.as_ref(),
        ctx.accounts.recipient3_ata.as_ref(),
    ];
    
    let recipient_pubkeys = [
        Some(ctx.accounts.recipient1.key()),
        ctx.accounts.recipient2.as_ref().map(|acc| acc.key()),
        ctx.accounts.recipient3.as_ref().map(|acc| acc.key()),
    ];
    
    let escrow_seed = &[
        b"token_escrow",
        ctx.accounts.escrow_account.base.buyer.as_ref(),
        ctx.accounts.escrow_account.base.seller.as_ref(),
        &[ctx.accounts.escrow_account.base.moderator.is_some() as u8],
        &ctx.accounts.escrow_account.base.unique_id,
        &[ctx.accounts.escrow_account.base.bump],
    ];
    
    // 验证支付金额
    verify_payment_amounts(&payment_amounts, &ctx.accounts.escrow_account.base)?;
    
    // 验证签名
    verify_signatures_without_timelock(
        &ctx.accounts.escrow_account.base,
        &signatures,
        &payment_amounts,
        &recipient_pubkeys,
        &ctx.accounts.sysvar_instructions,
    )?;
    
    // 转账代币
    transfer_tokens_to_recipients(
        &ctx,
        &payment_amounts,
        &recipient_accounts,
        escrow_seed
    )?;

    // 关闭代币账户
    token::close_account(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::CloseAccount {
                account: ctx.accounts.escrow_token_account.to_account_info(),
                destination: ctx.accounts.buyer.to_account_info(),
                authority: ctx.accounts.escrow_account.to_account_info(),
            },
            &[escrow_seed],
        )
    )?;
    
    // 关闭托管账户并返回租金
    close_escrow_and_return_rent(
        &ctx.accounts.escrow_account.to_account_info(),
        &ctx.accounts.buyer,
    )?;

    let id_hex = bytes_to_hex_string(&ctx.accounts.escrow_account.base.unique_id);

    msg!(
        "Token escrow completed: Buyer={}, Seller={}, ID=0x{}", 
        ctx.accounts.escrow_account.base.buyer, 
        ctx.accounts.escrow_account.base.seller,
        id_hex
    );

    Ok(())
}

fn transfer_tokens_to_recipients<'info>(
    ctx: &Context<ReleaseToken<'info>>,
    amounts: &[u64],
    recipients: &[Option<&Account<'info, TokenAccount>>],
    escrow_seed: &[&[u8]],
) -> Result<()> {
    for recipient in recipients.iter().flatten() {
        require!(
            recipient.mint == ctx.accounts.escrow_account.mint,
            EscrowError::TokenMintMismatch
        );
    }
    
    for (i, amount) in amounts.iter().enumerate() {
        if let Some(recipient) = recipients[i] {
            msg!("Transfer {} tokens to account {}",  amount, recipient.key());
            
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
    
    Ok(())
}

#[derive(Accounts)]
#[instruction(
    payment_amounts: Vec<u64>,
    signatures: Vec<Vec<u8>>
)]
pub struct ReleaseTokenAfterTimeout<'info> {
    #[account(mut)]
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
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,

    /// CHECK: Sysvar Instructions account
    #[account(address = solana_program::sysvar::instructions::ID)]
    pub sysvar_instructions: UncheckedAccount<'info>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub rent: Sysvar<'info, Rent>,
    pub token_mint: Account<'info, Mint>,

    /// CHECK: 买家账户
    #[account(mut, address = escrow_account.base.buyer @ EscrowError::ValidationFailed)]
    pub buyer: AccountInfo<'info>,

    pub clock: Sysvar<'info, Clock>,
    
    /// CHECK: 第一个接收方owner
    pub recipient1: AccountInfo<'info>,
    /// CHECK: 第一个接收方ATA
    #[account(
        init_if_needed,
        payer = initiator,
        associated_token::mint = token_mint,
        associated_token::authority = recipient1,
    )]
    pub recipient1_ata: Account<'info, TokenAccount>,
    
    /// CHECK: 第二个接收方owner
    pub recipient2: Option<AccountInfo<'info>>,
    /// CHECK: 第二个接收方ATA
    #[account(
        init_if_needed,
        payer = initiator,
        associated_token::mint = token_mint,
        associated_token::authority = recipient2,
    )]
    pub recipient2_ata: Option<Account<'info, TokenAccount>>,
}

fn transfer_tokens_to_recipients_after_timeout<'info>(
    ctx: &Context<ReleaseTokenAfterTimeout<'info>>,
    amounts: &[u64],
    recipients: &[Option<&Account<'info, TokenAccount>>],
    escrow_seed: &[&[u8]],
) -> Result<()> {
    for recipient in recipients.iter().flatten() {
        require!(
            recipient.mint == ctx.accounts.escrow_account.mint,
            EscrowError::TokenMintMismatch
        );
    }
    
    for (i, amount) in amounts.iter().enumerate() {
        if let Some(recipient) = recipients[i] {
            msg!("Transfer {} tokens to account {}", amount, recipient.key());
            
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
    
    Ok(())
}

pub fn handler_after_timeout(
    ctx: Context<ReleaseTokenAfterTimeout>,
    payment_amounts: Vec<u64>,
    signatures: Vec<Vec<u8>>
) -> Result<()> {
    // 验证接收方数量
    require!(
        payment_amounts.len() >= 1 && payment_amounts.len() <= 2,
        EscrowError::InvalidRecipientCount
    );
    
    let recipient_accounts = [
        Some(&ctx.accounts.recipient1_ata),
        ctx.accounts.recipient2_ata.as_ref(),
    ];
    
    let recipient_pubkeys = [
        Some(ctx.accounts.recipient1.key()),
        ctx.accounts.recipient2.as_ref().map(|acc| acc.key()),
    ];
    
    let escrow_seed = &[
        b"token_escrow",
        ctx.accounts.escrow_account.base.buyer.as_ref(),
        ctx.accounts.escrow_account.base.seller.as_ref(),
        &[ctx.accounts.escrow_account.base.moderator.is_some() as u8],
        &ctx.accounts.escrow_account.base.unique_id,
        &[ctx.accounts.escrow_account.base.bump],
    ];
    
    // 验证是否超时
    require!(
        ctx.accounts.clock.unix_timestamp >= ctx.accounts.escrow_account.base.unlock_time,
        EscrowError::TimelockNotExpired
    );
    
    // 验证支付金额
    verify_payment_amounts(&payment_amounts, &ctx.accounts.escrow_account.base)?;
    
    // 验证签名
    let message = construct_message(
        &ctx.accounts.escrow_account.base.unique_id,
        &recipient_pubkeys,
        &payment_amounts
    );
    
    let all_signers = verify_ed25519_instructions(
        &ctx.accounts.sysvar_instructions,
        &signatures,
        &message,
    )?;
    
    require!(
        all_signers.contains(&ctx.accounts.escrow_account.base.seller),
        EscrowError::InvalidSigner
    );
    
    // 转账代币
    transfer_tokens_to_recipients_after_timeout(
        &ctx,
        &payment_amounts,
        &recipient_accounts,
        escrow_seed
    )?;
    
    // 关闭代币账户
    token::close_account(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            token::CloseAccount {
                account: ctx.accounts.escrow_token_account.to_account_info(),
                destination: ctx.accounts.buyer.to_account_info(),
                authority: ctx.accounts.escrow_account.to_account_info(),
            },
            &[escrow_seed],
        )
    )?;
    
    // 关闭托管账户并返回租金
    close_escrow_and_return_rent(
        &ctx.accounts.escrow_account.to_account_info(),
        &ctx.accounts.buyer,
    )?;

    let id_hex = bytes_to_hex_string(&ctx.accounts.escrow_account.base.unique_id);

    msg!(
        "Token escrow completed after timeout: Buyer={}, Seller={}, ID=0x{}", 
        ctx.accounts.escrow_account.base.buyer, 
        ctx.accounts.escrow_account.base.seller,
        id_hex
    );

    Ok(())
} 