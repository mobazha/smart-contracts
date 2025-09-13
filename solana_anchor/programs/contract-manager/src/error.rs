use anchor_lang::prelude::*;

#[error_code]
pub enum ContractManagerError {
    #[msg("Contract name cannot be empty")]
    EmptyContractName,
    
    #[msg("Version name cannot be empty")]
    EmptyVersionName,
    
    #[msg("Contract already exists")]
    ContractAlreadyExists,
    
    #[msg("Version already exists for this contract")]
    VersionAlreadyExists,
    
    #[msg("Contract does not exist")]
    ContractNotFound,
    
    #[msg("Version does not exist for this contract")]
    VersionNotFound,
    
    #[msg("No recommended version set for this contract")]
    NoRecommendedVersion,
    
    #[msg("Invalid program ID")]
    InvalidProgramId,
    
    #[msg("Unauthorized access")]
    Unauthorized,
    
    #[msg("Contract manager not initialized")]
    NotInitialized,
}
