use anchor_lang::prelude::*;
use crate::{state::*, error::*};

#[derive(Accounts)]
pub struct Sign<'info> {
    #[account(
        mut,
        constraint = escrow_account.is_initialized @ EscrowError::InvalidAccountData,
        constraint = (
            (signer.key() == escrow_account.buyer && !escrow_account.buyer_signed) ||
            (signer.key() == escrow_account.seller && !escrow_account.seller_signed) ||
            (escrow_account.moderator.is_some() && signer.key() == escrow_account.moderator.unwrap() && !escrow_account.moderator_signed)
        ) @ EscrowError::AlreadySigned,
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
    
    pub signer: Signer<'info>,
}

pub fn handler(ctx: Context<Sign>) -> Result<()> {
    let escrow = &mut ctx.accounts.escrow_account;
    let signer_key = ctx.accounts.signer.key();
    
    // 更新对应签名方的签名状态
    if signer_key == escrow.buyer {
        escrow.buyer_signed = true;
        msg!("买家已签名");
    } else if signer_key == escrow.seller {
        escrow.seller_signed = true;
        msg!("卖家已签名");
    } else if escrow.moderator.is_some() && signer_key == escrow.moderator.unwrap() {
        escrow.moderator_signed = true;
        msg!("仲裁人已签名");
    }
    
    msg!("签名成功处理");
    
    Ok(())
} 