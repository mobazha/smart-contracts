use anchor_lang::prelude::*;
use crate::{state::*, error::*, utils::{close_escrow_and_return_rent, process_release, bytes_to_hex_string}};

#[derive(Accounts)]
#[instruction(
    payment_amounts: Vec<u64>,
    signatures: Vec<Vec<u8>>
)]
pub struct ReleaseSol<'info> {
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
            b"sol_escrow",
            escrow_account.base.buyer.as_ref(),
            escrow_account.base.seller.as_ref(),
            &[escrow_account.base.moderator.is_some() as u8],
            &escrow_account.base.unique_id
        ],
        bump = escrow_account.base.bump
    )]
    pub escrow_account: Account<'info, SolEscrow>,
    
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,

    /// CHECK: Sysvar Instructions account
    #[account(address = solana_program::sysvar::instructions::ID)]
    pub sysvar_instructions: UncheckedAccount<'info>,
    
    /// CHECK: 买家账户，必须是托管账户中指定的买家
    #[account(
        mut,
        constraint = buyer.key() == escrow_account.base.buyer @ EscrowError::ValidationFailed
    )]
    pub buyer: AccountInfo<'info>,
    
    /// CHECK: 第一个接收方账户
    #[account(mut)]
    pub recipient1: AccountInfo<'info>,
    
    /// CHECK: 第二个接收方账户
    #[account(mut)]
    pub recipient2: Option<AccountInfo<'info>>,
    
    /// CHECK: 第三个接收方账户
    #[account(mut)]
    pub recipient3: Option<AccountInfo<'info>>,
}

pub fn handler(
    ctx: Context<ReleaseSol>,
    payment_amounts: Vec<u64>,
    signatures: Vec<Vec<u8>>
) -> Result<()> {
    let recipient_pubkeys = [
        Some(ctx.accounts.recipient1.key()),
        ctx.accounts.recipient2.as_ref().map(|acc| acc.key()),
        ctx.accounts.recipient3.as_ref().map(|acc| acc.key()),
    ];
    
    process_release(
        &*ctx.accounts.escrow_account,
        &signatures,
        &payment_amounts,
        &recipient_pubkeys,
        ctx.accounts.clock.unix_timestamp,
        &ctx.accounts.sysvar_instructions,
        || {
            transfer_sol_to_recipients(&ctx, &payment_amounts, &recipient_pubkeys)?;
            
            close_escrow_and_return_rent(
                &ctx.accounts.escrow_account.to_account_info(),
                &ctx.accounts.buyer,
            )?;

            let id_hex = bytes_to_hex_string(&ctx.accounts.escrow_account.base.unique_id);

            msg!(
                "SOL escrow completed: Buyer={}, Seller={}, ID=0x{}", 
                ctx.accounts.escrow_account.base.buyer, 
                ctx.accounts.escrow_account.base.seller,
                id_hex
            );
            
            Ok(())
        },
    )
}

pub fn transfer_sol_to_recipients<'info>(
    ctx: &Context<crate::instructions::release_sol::ReleaseSol<'info>>,
    amounts: &[u64], 
    recipients: &[Option<Pubkey>]
) -> Result<()> {
    let escrow_info = ctx.accounts.escrow_account.to_account_info();
    let recipient1_info = ctx.accounts.recipient1.to_account_info();
    let recipient_accounts = [
        Some(&recipient1_info),  
        ctx.accounts.recipient2.as_ref(),
        ctx.accounts.recipient3.as_ref(),
    ];
    
    for (i, amount) in amounts.iter().enumerate() {
        if let Some(recipient_key) = recipients[i] {
            if let Some(recipient) = recipient_accounts[i] {
                require!(recipient.key() == recipient_key, EscrowError::InvalidRecipient);
                
                // 检查账户是否已初始化
                if recipient.lamports() == 0 {
                    // 创建账户，只支付必要的租金
                    anchor_lang::system_program::create_account(
                        CpiContext::new(
                            ctx.accounts.system_program.to_account_info(),
                            anchor_lang::system_program::CreateAccount {
                                from: ctx.accounts.initiator.to_account_info(),
                                to: recipient.clone(),
                            },
                        ),
                        ctx.accounts.rent.minimum_balance(0),  // 只支付必要的租金
                        0,
                        &ctx.accounts.system_program.key(),
                    )?;
                    
                    msg!(
                        "Created account {} and transferring {} lamports from escrow", 
                        recipient.key(),
                        *amount
                    );
                }

                let mut escrow_lamports = escrow_info.try_borrow_mut_lamports()?;
                let mut recipient_lamports = recipient.try_borrow_mut_lamports()?;
                
                require!(**escrow_lamports >= *amount, EscrowError::InsufficientFunds);

                msg!(
                    "Transfer {} lamports to account {}", 
                    amount, 
                    recipient.key()
                );
                
                **escrow_lamports -= amount;
                **recipient_lamports += amount;
            }
        }
    }
    
    Ok(())
}
