use anchor_lang::prelude::*;
use crate::error::*;

pub const MAX_PAYMENT_TARGETS: usize = 4;
pub const MAX_REQUIRED_SIGNATURES: u8 = 2;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EscrowAccount {
    pub is_initialized: bool,
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub moderator: Option<Pubkey>,
    pub payer_address: Pubkey, // address of the party who paid for the transaction
    pub required_signatures: u8,
    pub unlock_time: i64,
    pub unique_id: [u8; 20],
    pub amount: u64,
    pub bump: u8,
}

// SOL Escrow Account
#[account]
pub struct SolEscrow {
    pub base: EscrowAccount,
}

// Token Escrow Account
#[account]
pub struct TokenEscrow {
    pub base: EscrowAccount,
    pub mint: Pubkey
}

impl SolEscrow {
    pub const LEN: usize = 8 + // discriminator
                          1 + // is_initialized
                          32 + // buyer
                          32 + // seller
                          33 + // moderator (Option<Pubkey>)
                          32 + // payer_address
                          8 + // amount
                          8 + // unlock_time
                          1 + // required_signatures
                          20 + // unique_id
                          1;  // bump
}

impl TokenEscrow {
    pub const LEN: usize = 8 + // discriminator
                          1 + // is_initialized
                          32 + // buyer
                          32 + // seller
                          33 + // moderator (Option<Pubkey>)
                          32 + // payer_address
                          32 + // mint
                          8 + // amount
                          8 + // unlock_time
                          1 + // required_signatures
                          20 + // unique_id
                          1;  // bump
}

impl Default for SolEscrow {
    fn default() -> Self {
        Self {
            base: EscrowAccount {
                is_initialized: false,
                buyer: Pubkey::default(),
                seller: Pubkey::default(),
                moderator: None,
                payer_address: Pubkey::default(),
                required_signatures: 0,
                unlock_time: 0,
                unique_id: [0; 20],
                amount: 0,
                bump: 0,
            },
        }
    }
}

impl Default for TokenEscrow {
    fn default() -> Self {
        Self {
            base: EscrowAccount {
                is_initialized: false,
                buyer: Pubkey::default(),
                seller: Pubkey::default(),
                moderator: None,
                payer_address: Pubkey::default(),
                required_signatures: 0,
                unlock_time: 0,
                unique_id: [0; 20],
                amount: 0,
                bump: 0,
            },
            mint: Pubkey::default()
        }
    }
}

impl AsRef<EscrowAccount> for SolEscrow {
    fn as_ref(&self) -> &EscrowAccount {
        &self.base
    }
}

impl AsRef<EscrowAccount> for TokenEscrow {
    fn as_ref(&self) -> &EscrowAccount {
        &self.base
    }
}

impl EscrowAccount {
    pub fn new(
        buyer: Pubkey,
        seller: Pubkey,
        moderator: Option<Pubkey>,
        payer_address: Pubkey,
        required_signatures: u8,
        unlock_time: i64,
        unique_id: [u8; 20],
        amount: u64,
        bump: u8,
    ) -> Self {
        Self {
            is_initialized: true,
            buyer,
            seller,
            moderator,
            payer_address,
            required_signatures,
            unlock_time,
            unique_id,
            amount,
            bump,
        }
    }
    
    pub fn validate_required_signatures(&self) -> Result<()> {
        let max_possible = 2 + if self.moderator.is_some() { 1 } else { 0 };
        require!(
            self.required_signatures > 0,
            EscrowError::InvalidRequiredSignatures
        );
        require!(
            self.required_signatures <= max_possible,
            EscrowError::InvalidRequiredSignatures
        );
        Ok(())
    }
} 