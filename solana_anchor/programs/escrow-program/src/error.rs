use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Invalid instruction")]
    InvalidInstruction,
    
    #[msg("Escrow already completed")]
    AlreadyCompleted,
    
    #[msg("Insufficient signatures")]
    InsufficientSignatures,
    
    #[msg("Invalid signer")]
    InvalidSigner,
    
    #[msg("Invalid account data")]
    InvalidAccountData,
    
    #[msg("Amount overflow")]
    AmountOverflow,
    
    #[msg("Invalid token account")]
    InvalidTokenAccount,
    
    #[msg("Amount cannot be zero")]
    ZeroAmount,
    
    #[msg("Payment targets list is empty")]
    EmptyPaymentTargets,
    
    #[msg("Too many payment targets")]
    TooManyPaymentTargets,
    
    #[msg("Too many required signatures")]
    TooManyRequiredSignatures,
} 