use anchor_lang::prelude::*;
use crate::state::ContractManager;
use crate::error::ContractManagerError;

#[derive(Accounts)]
pub struct RemoveRecommended<'info> {
    #[account(
        mut,
        seeds = [b"contract_manager"],
        bump = contract_manager.bump,
        has_one = authority
    )]
    pub contract_manager: Account<'info, ContractManager>,
    
    pub authority: Signer<'info>,
}

pub fn handler(
    ctx: Context<RemoveRecommended>,
    contract_name: String,
) -> Result<()> {
    let contract_manager = &mut ctx.accounts.contract_manager;
    
    // Find the contract
    let contract = contract_manager
        .find_contract_mut(&contract_name)
        .ok_or(ContractManagerError::ContractNotFound)?;

    // Remove recommended version
    contract.recommended_version = None;

    msg!(
        "Removed recommended version for contract {}",
        contract_name
    );

    Ok(())
}
