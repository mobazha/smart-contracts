use solana_program_error::ProgramError;

#[derive(Debug, thiserror::Error)]
pub enum EscrowError {
    #[error("Invalid instruction")]
    InvalidInstruction,
    
    #[error("Escrow already completed")]
    AlreadyCompleted,
    
    #[error("Insufficient signatures")]
    InsufficientSignatures,
    
    #[error("Invalid signer")]
    InvalidSigner,
    
    #[error("Invalid account data")]
    InvalidAccountData,
    
    #[error("Amount overflow")]
    AmountOverflow,
    
    #[error("Invalid token account")]
    InvalidTokenAccount,
    
    #[error("Amount cannot be zero")]
    ZeroAmount,
    
    #[error("Payment targets list is empty")]
    EmptyPaymentTargets,
}

impl From<EscrowError> for ProgramError {
    fn from(e: EscrowError) -> Self {
        ProgramError::Custom(e as u32)
    }
} 