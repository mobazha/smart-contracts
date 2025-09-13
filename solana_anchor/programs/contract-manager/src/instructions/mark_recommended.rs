use anchor_lang::prelude::*;
use crate::state::ContractManager;
use crate::error::ContractManagerError;

#[derive(Accounts)]
pub struct MarkRecommended<'info> {
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
    ctx: Context<MarkRecommended>,
    contract_name: String,
    version_name: String,
) -> Result<()> {
    let contract_manager = &mut ctx.accounts.contract_manager;
    
    // Find the contract
    let contract = contract_manager
        .find_contract_mut(&contract_name)
        .ok_or(ContractManagerError::ContractNotFound)?;

    // Verify the version exists
    require!(
        contract.versions.iter().any(|v| v.version_name == version_name),
        ContractManagerError::VersionNotFound
    );

    // Set as recommended version
    contract.recommended_version = Some(version_name.clone());

    msg!(
        "Marked version {} as recommended for contract {}",
        version_name,
        contract_name
    );

    Ok(())
}
