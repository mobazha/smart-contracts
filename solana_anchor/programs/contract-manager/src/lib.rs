use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod error;

use state::{ContractStatus, BugLevel};

use instructions::{
    add_version::*,
    update_version::*,
    mark_recommended::*,
    remove_recommended::*,
    initialize::*,
};

declare_id!("6LmWMjAMAfVdc8mpgPjHvFLa2sbcudiLiJT3bAGRYMMD");

#[program]
pub mod contract_manager {
    use super::*;

    /// Initialize the contract manager
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize::handler(ctx)
    }

    /// Add a new version of a contract
    pub fn add_version(
        ctx: Context<AddVersion>,
        contract_name: String,
        version_name: String,
        status: ContractStatus,
        program_id: Pubkey,
    ) -> Result<()> {
        instructions::add_version::handler(
            ctx,
            contract_name,
            version_name,
            status,
            program_id,
        )
    }

    /// Update an existing version
    pub fn update_version(
        ctx: Context<UpdateVersion>,
        contract_name: String,
        version_name: String,
        status: ContractStatus,
        bug_level: BugLevel,
    ) -> Result<()> {
        instructions::update_version::handler(
            ctx,
            contract_name,
            version_name,
            status,
            bug_level,
        )
    }

    /// Mark a version as recommended
    pub fn mark_recommended(
        ctx: Context<MarkRecommended>,
        contract_name: String,
        version_name: String,
    ) -> Result<()> {
        instructions::mark_recommended::handler(ctx, contract_name, version_name)
    }

    /// Remove recommended version
    pub fn remove_recommended(
        ctx: Context<RemoveRecommended>,
        contract_name: String,
    ) -> Result<()> {
        instructions::remove_recommended::handler(ctx, contract_name)
    }
}
