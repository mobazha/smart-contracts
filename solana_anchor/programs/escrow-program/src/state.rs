use anchor_lang::prelude::*;
use crate::error::*;

// 定义常量
pub const ESCROW_SEED_PREFIX: &[u8] = b"escrow";
pub const MAX_PAYMENT_TARGETS: usize = 4;
pub const MAX_REQUIRED_SIGNATURES: u8 = 2;

// 将 BaseEscrow 重命名为 EscrowAccount
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EscrowAccount {
    pub is_initialized: bool,
    pub buyer: Pubkey,
    pub seller: Pubkey,
    pub moderator: Option<Pubkey>,
    pub required_signatures: u8,
    pub unlock_time: i64,
    pub unique_id: [u8; 20],  // 可以减少为16字节或8字节以进一步优化
    pub amount: u64,
    pub bump: u8,
}

// SOL托管账户
#[account]
pub struct SolEscrow {
    // 使用基础托管结构
    pub base: EscrowAccount,
}

// 代币托管账户
#[account]
pub struct TokenEscrow {
    // 使用基础托管结构
    pub base: EscrowAccount,
    // 代币特有字段
    pub mint: Pubkey
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
            base: EscrowAccount {
                is_initialized: false,
                buyer: Pubkey::default(),
                seller: Pubkey::default(),
                moderator: None,
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

// 更新 AsRef 实现
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

// 实用方法现在在 EscrowAccount 上
impl EscrowAccount {
    pub fn new(
        buyer: Pubkey,
        seller: Pubkey,
        moderator: Option<Pubkey>,
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
            required_signatures,
            unlock_time,
            unique_id,
            amount,
            bump,
        }
    }
    
    pub fn validate_required_signatures(&self) -> Result<()> {
        // 基本验证逻辑
        let max_possible = 2 + if self.moderator.is_some() { 1 } else { 0 };
        require!(
            self.required_signatures > 0 && self.required_signatures <= max_possible,
            EscrowError::ValidationFailed
        );
        Ok(())
    }
} 