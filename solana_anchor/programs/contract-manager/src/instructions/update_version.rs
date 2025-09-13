use anchor_lang::prelude::*;
use crate::state::{ContractManager, ContractStatus, BugLevel};
use crate::error::ContractManagerError;

#[derive(Accounts)]
pub struct UpdateVersion<'info> {
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
    ctx: Context<UpdateVersion>,
    contract_name: String,
    version_name: String,
    status: ContractStatus,
    bug_level: BugLevel,
) -> Result<()> {
    let contract_manager = &mut ctx.accounts.contract_manager;
    
    // Find the contract
    let contract = contract_manager
        .find_contract_mut(&contract_name)
        .ok_or(ContractManagerError::ContractNotFound)?;

    // Find the version
    let version = contract
        .versions
        .iter_mut()
        .find(|v| v.version_name == version_name)
        .ok_or(ContractManagerError::VersionNotFound)?;

    // Update the version
    version.status = status;
    version.bug_level = bug_level;

    msg!(
        "Updated version {} for contract {} with new status and bug level",
        version_name,
        contract_name
    );

    Ok(())
}
