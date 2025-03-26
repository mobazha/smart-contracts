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
pub struct Escrow {
    pub is_initialized: bool,              // 1 byte - 表示账户是否初始化
    pub buyer: Pubkey,                     // 32 bytes
    pub seller: Pubkey,                    // 32 bytes
    pub moderator: Option<Pubkey>,         // 33 bytes
    pub token_type: TokenType,             // 33 bytes
    pub amount: u64,                       // 8 bytes
    pub unlock_time: i64,                  // 8 bytes
    pub required_signatures: u8,           // 1 byte
    pub unique_id: [u8; 20],               // 20 bytes
    pub bump: u8,
}

impl Escrow {
    pub const LEN: usize = 8 + // 判别器
                          1 + // is_initialized
                          32 + // buyer
                          32 + // seller
                          33 + // moderator (Option<Pubkey>)
                          33 + // token_type
                          8 + // amount
                          8 + // unlock_time
                          1 + // required_signatures
                          20 + // unique_id
                          1;  // bump
}

impl Default for Escrow {
    fn default() -> Self {
        Self {
            is_initialized: false,
            buyer: Pubkey::default(),
            seller: Pubkey::default(),
            moderator: None,
            token_type: TokenType::Sol,
            amount: 0,
            unlock_time: 0,
            required_signatures: 0,
            unique_id: [0; 20],
            bump: 0,
        }
    }
} 