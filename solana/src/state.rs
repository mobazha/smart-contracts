use borsh::{BorshDeserialize, BorshSerialize};
use solana_program_pack::{IsInitialized, Sealed};
use solana_pubkey::Pubkey;
use solana_program_error::ProgramError;

#[derive(BorshSerialize, BorshDeserialize, Debug, PartialEq)]
pub enum EscrowState {
    Uninitialized,
    Active,
    Completed,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct PaymentTarget {
    pub recipient: Pubkey,
    pub amount: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
pub enum TokenType {
    Sol,
    Spl(Pubkey),  // Pubkey 是代币的 mint 地址
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Escrow {
    pub state: EscrowState,           // 1 byte
    pub buyer: Pubkey,                // 32 bytes
    pub seller: Pubkey,               // 32 bytes
    pub moderator: Option<Pubkey>,    // 33 bytes
    pub token_type: TokenType,        // ~33 bytes
    pub amount: u64,                  // 8 bytes
    pub unlock_time: i64,             // 8 bytes
    pub required_signatures: u8,       // 1 byte
    pub buyer_signed: bool,           // 1 byte
    pub seller_signed: bool,          // 1 byte
    pub moderator_signed: bool,       // 1 byte
    pub is_initialized: bool,         // 1 byte
    pub unique_id: [u8; 20],         // 20 bytes
    pub bump_seed: u8,                // 1 byte
}

impl Sealed for Escrow {}

impl IsInitialized for Escrow {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}

impl Escrow {
    pub const LEN: usize = 256; // 当前使用约 152 字节，预留约 100 字节供扩展
    
    pub fn pack_into_slice(&self, dst: &mut [u8]) {
        let data = self.try_to_vec().unwrap();
        dst[..data.len()].copy_from_slice(&data);
    }
    
    pub fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        Self::try_from_slice(src).map_err(|_| ProgramError::InvalidAccountData)
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
            bump_seed: 0,
        }
    }
}

impl Escrow {
    pub fn get_escrow_address(
        program_id: &Pubkey,
        buyer: &Pubkey,
        seller: &Pubkey,
        moderator: Option<&Pubkey>,
        unique_id: [u8; 20],  // 修改参数名
    ) -> (Pubkey, u8) {
        let seeds = if let Some(moderator) = moderator {
            vec![
                ESCROW_SEED_PREFIX,
                buyer.as_ref(),
                seller.as_ref(),
                moderator.as_ref(),
                &unique_id,
            ]
        } else {
            vec![
                ESCROW_SEED_PREFIX,
                buyer.as_ref(),
                seller.as_ref(),
                &unique_id,
            ]
        };

        Pubkey::find_program_address(&seeds[..], program_id)
    }
}

// 添加程序常量
pub const ESCROW_SEED_PREFIX: &[u8] = b"escrow";