use anchor_lang::prelude::*;

// 定义常量
pub const ESCROW_SEED_PREFIX: &[u8] = b"escrow";
pub const MAX_PAYMENT_TARGETS: usize = 4;
pub const MAX_REQUIRED_SIGNATURES: u8 = 2;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Debug)]
pub enum EscrowState {
    Uninitialized,
    Active,
    Completed,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PaymentTarget {
    pub recipient: Pubkey,
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum TokenType {
    Sol,
    Spl(Pubkey),  // Pubkey 是代币的 mint 地址
}

#[account]
#[derive(Debug)]
pub struct Escrow {
    pub state: EscrowState,           // 1 byte
    pub buyer: Pubkey,                // 32 bytes
    pub seller: Pubkey,               // 32 bytes
    pub moderator: Option<Pubkey>,    // 33 bytes
    pub token_type: TokenType,        // ~33 bytes
    pub amount: u64,                  // 8 bytes
    pub unlock_time: i64,             // 8 bytes
    pub required_signatures: u8,      // 1 byte
    pub buyer_signed: bool,           // 1 byte
    pub seller_signed: bool,          // 1 byte
    pub moderator_signed: bool,       // 1 byte
    pub is_initialized: bool,         // 1 byte
    pub unique_id: [u8; 20],          // 20 bytes
}

impl Escrow {
    pub const LEN: usize = 256;

    pub fn get_escrow_address(
        program_id: &Pubkey,
        buyer: &Pubkey,
        seller: &Pubkey,
        moderator: Option<&Pubkey>,
        unique_id: [u8; 20],
    ) -> (Pubkey, u8) {
        let moderator_ref = moderator.map(|m| m.as_ref()).unwrap_or(&[]);
        
        let seeds = if moderator.is_some() {
            &[
                ESCROW_SEED_PREFIX,
                buyer.as_ref(),
                seller.as_ref(),
                moderator_ref,
                &unique_id[..],
            ][..]
        } else {
            &[
                ESCROW_SEED_PREFIX,
                buyer.as_ref(),
                seller.as_ref(),
                &unique_id[..],
            ][..]
        };

        Pubkey::find_program_address(seeds, program_id)
    }
}

impl Default for Escrow {
    fn default() -> Self {
        Self {
            state: EscrowState::Uninitialized,
            buyer: Pubkey::default(),
            seller: Pubkey::default(),
            moderator: None,
            token_type: TokenType::Sol,
            amount: 0,
            unlock_time: 0,
            required_signatures: 0,
            buyer_signed: false,
            seller_signed: false,
            moderator_signed: false,
            is_initialized: false,
            unique_id: [0; 20],
        }
    }
} 