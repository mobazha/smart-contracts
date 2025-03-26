use anchor_lang::prelude::*;

// 定义常量
pub const ESCROW_SEED_PREFIX: &[u8] = b"escrow";
pub const MAX_PAYMENT_TARGETS: usize = 4;
pub const MAX_REQUIRED_SIGNATURES: u8 = 2;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub enum TokenType {
    #[default] // 标记默认变体
    Sol,
    Spl { mint: Pubkey },
}

#[account]
pub struct SolEscrow {
    pub is_initialized: bool,              // 1 byte - 表示账户是否初始化
    pub buyer: Pubkey,                     // 32 bytes
    pub seller: Pubkey,                    // 32 bytes
    pub moderator: Option<Pubkey>,         // 33 bytes
    pub amount: u64,                       // 8 bytes
    pub unlock_time: i64,                  // 8 bytes
    pub required_signatures: u8,           // 1 byte
    pub unique_id: [u8; 20],               // 20 bytes
    pub bump: u8,
}

#[account]
pub struct TokenEscrow {
    pub is_initialized: bool,
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub moderator: Option<Pubkey>,
    pub mint: Pubkey,
    pub amount: u64,
    pub unlock_time: i64,
    pub required_signatures: u8,
    pub unique_id: [u8; 20],
    pub bump: u8,
}

impl SolEscrow {
    pub const LEN: usize = 8 + // 判别器
                          1 + // is_initialized
                          32 + // buyer
                          32 + // seller
                          33 + // moderator (Option<Pubkey>)
                          8 + // amount
                          8 + // unlock_time
                          1 + // required_signatures
                          20 + // unique_id
                          1;  // bump
}

impl TokenEscrow {
    pub const LEN: usize = 8 + // 判别器
                          1 + // is_initialized
                          32 + // buyer
                          32 + // seller
                          33 + // moderator (Option<Pubkey>)
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
            is_initialized: false,
            buyer: Pubkey::default(),
            seller: Pubkey::default(),
            moderator: None,
            amount: 0,
            unlock_time: 0,
            required_signatures: 0,
            unique_id: [0; 20],
            bump: 0,
        }
    }
}

impl Default for TokenEscrow {
    fn default() -> Self {
        Self {
            is_initialized: false,
            buyer: Pubkey::default(),
            seller: Pubkey::default(),
            moderator: None,
            mint: Pubkey::default(),
            amount: 0,
            unlock_time: 0,
            required_signatures: 0,
            unique_id: [0; 20],
            bump: 0,
        }
    }
} 