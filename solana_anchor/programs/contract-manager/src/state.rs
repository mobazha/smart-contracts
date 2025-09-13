use anchor_lang::prelude::*;

/// Contract status enum
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum ContractStatus {
    Beta,
    ReleaseCandidate,
    Production,
    Deprecated,
}

/// Bug level enum
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum BugLevel {
    None,
    Low,
    Medium,
    High,
    Critical,
}

/// Version information struct
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Version {
    pub version_name: String,
    pub status: ContractStatus,
    pub bug_level: BugLevel,
    pub program_id: Pubkey,
    pub date_added: i64,
}

/// Contract information struct
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Contract {
    pub contract_name: String,
    pub versions: Vec<Version>,
    pub recommended_version: Option<String>,
}

/// Contract Manager state
#[account]
pub struct ContractManager {
    pub authority: Pubkey,
    pub contracts: Vec<Contract>,
    pub bump: u8,
}

impl ContractManager {
    pub const LEN: usize = 8 + // discriminator
                          32 + // authority
                          4 + // contracts vector length
                          1024 + // contracts data (estimated)
                          1; // bump

    /// Find a contract by name
    pub fn find_contract(&self, contract_name: &str) -> Option<&Contract> {
        self.contracts.iter().find(|c| c.contract_name == contract_name)
    }

    /// Find a contract by name (mutable)
    pub fn find_contract_mut(&mut self, contract_name: &str) -> Option<&mut Contract> {
        self.contracts.iter_mut().find(|c| c.contract_name == contract_name)
    }

    /// Add a new contract
    pub fn add_contract(&mut self, contract: Contract) {
        self.contracts.push(contract);
    }

    /// Get recommended version for a contract
    pub fn get_recommended_version(&self, contract_name: &str) -> Option<&Version> {
        if let Some(contract) = self.find_contract(contract_name) {
            if let Some(ref recommended_name) = contract.recommended_version {
                contract.versions.iter().find(|v| v.version_name == *recommended_name)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Get all versions for a contract
    pub fn get_versions(&self, contract_name: &str) -> Option<&Vec<Version>> {
        self.find_contract(contract_name).map(|c| &c.versions)
    }
}

impl Default for ContractManager {
    fn default() -> Self {
        Self {
            authority: Pubkey::default(),
            contracts: Vec::new(),
            bump: 0,
        }
    }
}
