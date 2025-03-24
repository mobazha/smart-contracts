use anchor_lang::prelude::*;
use crate::{state::*, error::*};

#[derive(Accounts)]
pub struct Sign<'info> {
    pub signer: Signer<'info>,
    
    #[account(
        mut,
        constraint = escrow_account.state == EscrowState::Active @ EscrowError::AlreadyCompleted,
        constraint = escrow_account.is_initialized @ EscrowError::InvalidAccountData
    )]
    pub escrow_account: Account<'info, Escrow>,
}

pub fn handler(ctx: Context<Sign>) -> Result<()> {
    let escrow = &mut ctx.accounts.escrow_account;
    let signer_key = ctx.accounts.signer.key();
    
    // 更新对应角色的签名状态
    if signer_key == escrow.buyer {
        escrow.buyer_signed = true;
    } else if signer_key == escrow.seller {
        escrow.seller_signed = true;
    } else if let Some(moderator) = escrow.moderator {
        if signer_key == moderator {
            escrow.moderator_signed = true;
        } else {
            return err!(EscrowError::InvalidSigner);
        }
    } else {
        return err!(EscrowError::InvalidSigner);
    }
    
    Ok(())
} 