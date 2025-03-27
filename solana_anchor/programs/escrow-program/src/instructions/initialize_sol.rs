use anchor_lang::prelude::*;
use crate::state::*;

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
    let escrow = &mut ctx.accounts.escrow_account;
    
    // 初始化基础托管账户
    escrow.base = EscrowAccount::new(
        ctx.accounts.buyer.key(),
        ctx.accounts.seller.key(),
        moderator,
        required_signatures,
        ctx.accounts.clock.unix_timestamp + (unlock_hours * 3600) as i64,
        unique_id,
        amount,
        ctx.bumps.escrow_account,
    );
    
    // 验证参数
    escrow.base.validate_required_signatures()?;
    
    // 转移 SOL 到托管账户
    anchor_lang::system_program::transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            anchor_lang::system_program::Transfer {
                from: ctx.accounts.buyer.to_account_info(),
                to: ctx.accounts.escrow_account.to_account_info(),
            },
        ),
        amount,
    )?;
    
    msg!("SOL托管已初始化，金额: {} lamports", amount);
    
    Ok(())
} 