use anchor_lang::prelude::*;
use crate::state::{ContractManager, Contract, Version, ContractStatus, BugLevel};
use crate::error::ContractManagerError;

#[derive(Accounts)]
pub struct AddVersion<'info> {
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
    ctx: Context<AddVersion>,
    contract_name: String,
    version_name: String,
    status: ContractStatus,
    program_id: Pubkey,
) -> Result<()> {
    // Validate inputs
    require!(!contract_name.is_empty(), ContractManagerError::EmptyContractName);
    require!(!version_name.is_empty(), ContractManagerError::EmptyVersionName);
    require!(program_id != Pubkey::default(), ContractManagerError::InvalidProgramId);

    let contract_manager = &mut ctx.accounts.contract_manager;
    
    // Check if contract exists, if not create it
    let contract_index = if let Some(index) = contract_manager.contracts.iter().position(|c| c.contract_name == contract_name) {
        index
    } else {
        // Create new contract
        let new_contract = Contract {
            contract_name: contract_name.clone(),
            versions: Vec::new(),
            recommended_version: None,
        };
        contract_manager.contracts.push(new_contract);
        contract_manager.contracts.len() - 1
    };

    let contract = &mut contract_manager.contracts[contract_index];
    
    // Check if version already exists
    require!(
        !contract.versions.iter().any(|v| v.version_name == version_name),
        ContractManagerError::VersionAlreadyExists
    );

    // Create new version
    let new_version = Version {
        version_name: version_name.clone(),
        status,
        bug_level: BugLevel::None,
        program_id,
        date_added: Clock::get()?.unix_timestamp,
    };

    contract.versions.push(new_version);

    msg!(
        "Added version {} for contract {} with program ID: {}",
        version_name,
        contract_name,
        program_id
    );

    Ok(())
}
