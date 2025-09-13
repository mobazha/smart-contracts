use anchor_lang::prelude::*;
use crate::state::ContractManager;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = ContractManager::LEN,
        seeds = [b"contract_manager"],
        bump
    )]
    pub contract_manager: Account<'info, ContractManager>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    let contract_manager = &mut ctx.accounts.contract_manager;
    contract_manager.authority = ctx.accounts.authority.key();
    contract_manager.contracts = Vec::new();
    contract_manager.bump = ctx.bumps.contract_manager;
    
    msg!("Contract manager initialized with authority: {}", ctx.accounts.authority.key());
    
    Ok(())
}
