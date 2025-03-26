use anchor_lang::prelude::*;

// 定义常量
pub const ESCROW_SEED_PREFIX: &[u8] = b"escrow";
pub const MAX_PAYMENT_TARGETS: usize = 4;
pub const MAX_REQUIRED_SIGNATURES: u8 = 2;

// 添加记录账户模式的常量
pub const POOL_SEED: &[u8] = b"escrow_pool";
pub const RECORD_SEED: &[u8] = b"escrow_record";

// 新增：资金池账户
#[account]
pub struct FundPool {
    pub authority: Pubkey,       // 资金池管理者（平台）
    pub total_sol_balance: u64,  // SOL总余额
    pub total_transactions: u64, // 交易总数
    pub is_active: bool,         // 资金池是否活跃
    pub bump: u8,                // PDA bump
}

// 代币池账户，每种代币一个池子
#[account]
pub struct TokenPool {
    pub authority: Pubkey,       // 资金池管理者（平台）
    pub mint: Pubkey,            // 代币铸造地址
    pub token_account: Pubkey,   // 代币账户
    pub total_balance: u64,      // 代币总余额
    pub total_transactions: u64, // 交易总数
    pub is_active: bool,         // 资金池是否活跃
    pub bump: u8,                // PDA bump
}

// 交易状态枚举
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum TransactionStatus {
    Pending,    // 等待中
    Completed,  // 已完成
    Cancelled,  // 已取消
    Disputed,   // 有争议
}

// 交易记录账户 - SOL版本
#[account]
pub struct SolEscrowRecord {
    pub buyer: Pubkey,                // 买家
    pub seller: Pubkey,               // 卖家
    pub moderator: Option<Pubkey>,    // 仲裁人(可选)
    pub amount: u64,                  // 交易金额
    pub unlock_time: i64,             // 解锁时间
    pub required_signatures: u8,      // 所需签名数
    pub unique_id: [u8; 20],          // 唯一ID
    pub status: TransactionStatus,    // 交易状态
    pub creation_time: i64,           // 创建时间
    pub completion_time: Option<i64>, // 完成时间
    pub bump: u8,                     // PDA bump
}

// 交易记录账户 - 代币版本
#[account]
pub struct TokenEscrowRecord {
    pub buyer: Pubkey,                // 买家
    pub seller: Pubkey,               // 卖家
    pub moderator: Option<Pubkey>,    // 仲裁人(可选)
    pub mint: Pubkey,                 // 代币铸造地址
    pub amount: u64,                  // 交易金额
    pub unlock_time: i64,             // 解锁时间
    pub required_signatures: u8,      // 所需签名数
    pub unique_id: [u8; 20],          // 唯一ID
    pub status: TransactionStatus,    // 交易状态
    pub creation_time: i64,           // 创建时间
    pub completion_time: Option<i64>, // 完成时间
    pub bump: u8,                     // PDA bump
}

// 为新账户结构定义常量大小
impl FundPool {
    pub const LEN: usize = 8 +  // 判别器
                          32 +   // authority
                          8 +    // total_sol_balance
                          8 +    // total_transactions
                          1 +    // is_active
                          1;     // bump
}

impl TokenPool {
    pub const LEN: usize = 8 +  // 判别器
                          32 +   // authority
                          32 +   // mint
                          32 +   // token_account
                          8 +    // total_balance
                          8 +    // total_transactions
                          1 +    // is_active
                          1;     // bump
}

impl SolEscrowRecord {
    pub const LEN: usize = 8 +  // 判别器
                          32 +   // buyer
                          32 +   // seller
                          33 +   // moderator (Option<Pubkey>)
                          8 +    // amount
                          8 +    // unlock_time
                          1 +    // required_signatures
                          20 +   // unique_id
                          2 +    // status (枚举)
                          8 +    // creation_time
                          9 +    // completion_time (Option<i64>)
                          1;     // bump
}

impl TokenEscrowRecord {
    pub const LEN: usize = 8 +  // 判别器
                          32 +   // buyer
                          32 +   // seller
                          33 +   // moderator (Option<Pubkey>)
                          32 +   // mint
                          8 +    // amount
                          8 +    // unlock_time
                          1 +    // required_signatures
                          20 +   // unique_id
                          2 +    // status (枚举)
                          8 +    // creation_time
                          9 +    // completion_time (Option<i64>)
                          1;     // bump
}

// 保留旧的账户结构以兼容现有数据
#[account]
pub struct SolEscrow {
    pub is_initialized: bool,
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub moderator: Option<Pubkey>,
    pub amount: u64,
    pub unlock_time: i64,
    pub required_signatures: u8,
    pub unique_id: [u8; 20],
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

// 实现默认特征
impl Default for TransactionStatus {
    fn default() -> Self {
        TransactionStatus::Pending
    }
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