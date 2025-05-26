use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Invalid Operation")]
    InvalidOperation,
    
    #[msg("Validation Failed")]
    ValidationFailed,
    
    #[msg("Signature Verification Failed")]
    SignatureVerificationFailed,
    
    #[msg("Escrow Account Already Exists")]
    EscrowAlreadyExists,
    
    #[msg("Invalid Payment Parameters")]
    InvalidPaymentParameters,
    
    #[msg("Unauthorized Operation")]
    Unauthorized,
    
    #[msg("Invalid Instruction Format")]
    InvalidInstruction,
    
    #[msg("Insufficient Signatures")]
    InsufficientSignatures,
    
    #[msg("Signer Is Not Authorized")]
    InvalidSigner,
    
    #[msg("Insufficient Funds In Escrow")]
    InsufficientFunds,
    
    #[msg("Invalid Recipient Account")]
    InvalidRecipient,
    
    #[msg("Too Many Recipients")]
    TooManyRecipients,
    
    #[msg("Total Payment Amount Exceeds Escrow Balance")]
    PaymentAmountExceedsEscrow,
    
    #[msg("Payment Amount Cannot Be Zero")]
    ZeroPaymentAmount,
    
    #[msg("Token Mint Mismatch")]
    TokenMintMismatch,
    
    #[msg("Invalid Required Signatures Count")]
    InvalidRequiredSignatures,
    
    #[msg("Ed25519 Instruction Not Found Or Invalid")]
    InvalidEd25519Instruction,

    #[msg("Account Is Not Initialized")]
    AccountNotInitialized,

    #[msg("Invalid Amount")]
    InvalidAmount,

    #[msg("接收方数量无效")]
    InvalidRecipientCount,

    #[msg("Timelock has not expired yet")]
    TimelockNotExpired,
} 