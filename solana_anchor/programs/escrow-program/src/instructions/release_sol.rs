use anchor_lang::prelude::*;
use crate::{state::*, error::*, utils::{verify_payment_amounts, verify_signatures_with_timelock, close_escrow_and_return_rent}};

#[derive(Accounts)]
#[instruction(
    payment_amounts: Vec<u64>,
    signatures: Vec<Vec<u8>>
)]
pub struct ReleaseSol<'info> {
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
            b"sol_escrow",
            escrow_account.buyer.as_ref(),
            escrow_account.seller.as_ref(),
            &[escrow_account.moderator.is_some() as u8],
            &escrow_account.unique_id
        ],
        bump = escrow_account.bump
    )]
    pub escrow_account: Account<'info, SolEscrow>,
    
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
    
    /// CHECK: 买家账户
    #[account(mut)]
    pub buyer: AccountInfo<'info>,
    
    /// CHECK: 接收方账户会在指令中验证
    #[account(mut)]
    pub recipient1: UncheckedAccount<'info>,
    
    /// CHECK: 第二个接收方账户，如果有的话
    #[account(mut)]
    pub recipient2: Option<AccountInfo<'info>>,
    
    /// CHECK: 第三个接收方账户，如果有的话
    #[account(mut)]
    pub recipient3: Option<AccountInfo<'info>>,
    
    /// CHECK: 第四个接收方账户，如果有的话
    #[account(mut)]
    pub recipient4: Option<AccountInfo<'info>>,
}

pub fn handler(
    ctx: Context<ReleaseSol>,
    payment_amounts: Vec<u64>,
    signatures: Vec<Vec<u8>>
) -> Result<()> {
    // 验证支付金额
    verify_payment_amounts(&payment_amounts, &ctx.accounts.escrow_account)?;
    
    // 验证签名（包含时间锁检查）
    verify_signatures_with_timelock(
        &*ctx.accounts.escrow_account,
        &signatures,
        &payment_amounts,
        ctx.accounts.clock.unix_timestamp,
    )?;
    
    // 执行支付
    let recipients = vec![
        Some(ctx.accounts.recipient1.to_account_info()),
        ctx.accounts.recipient2.as_ref().map(|r| r.to_account_info()),
        ctx.accounts.recipient3.as_ref().map(|r| r.to_account_info()),
        ctx.accounts.recipient4.as_ref().map(|r| r.to_account_info()),
    ];
    
    process_sol_payments(&payment_amounts, &recipients, &ctx.accounts.escrow_account)?;
    
    // 关闭托管账户并返回租金
    close_escrow_and_return_rent(
        &ctx.accounts.escrow_account.to_account_info(),
        &ctx.accounts.buyer,
    )?;
    
    Ok(())
}

fn process_sol_payments(
    amounts: &[u64],
    recipients: &[Option<AccountInfo>],
    escrow_account: &Account<SolEscrow>
) -> Result<()> {
    let escrow_info = escrow_account.to_account_info();
    
    // 遍历所有支付目标
    for (i, amount) in amounts.iter().enumerate() {
        if let Some(recipient) = &recipients[i] {
            let mut escrow_lamports = escrow_info.try_borrow_mut_lamports()?;
            let mut recipient_lamports = recipient.try_borrow_mut_lamports()?;
            
            require!(**escrow_lamports >= *amount, EscrowError::InsufficientFunds);
            
            // 添加转账日志
            msg!(
                "Transfer {} lamports to account {}", 
                amount, 
                recipient.key()
            );
            
            **escrow_lamports -= amount;
            **recipient_lamports += amount;
        }
    }
    
    // 添加完成日志
    msg!(
        "SOL escrow completed: Buyer={}, Seller={}, ID={:?}", 
        escrow_account.buyer, 
        escrow_account.seller,
        escrow_account.unique_id
    );
    
    Ok(())
} 