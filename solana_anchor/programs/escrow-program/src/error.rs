use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    // 通用错误
    #[msg("无效操作")]
    InvalidOperation,
    
    // 合并多种验证错误
    #[msg("验证失败")]
    ValidationFailed,
    
    // 签名相关错误（合并多种签名错误）
    #[msg("签名验证失败")]
    SignatureVerificationFailed,
    
    // 托管账户错误
    #[msg("托管账户已存在")]
    EscrowAlreadyExists,
    
    // 付款错误（合并金额和接收方错误）
    #[msg("付款参数无效")]
    InvalidPaymentParameters,
    
    // 操作权限错误
    #[msg("无操作权限")]
    Unauthorized,
    
    // 时间锁错误
    #[msg("时间锁未到期")]
    TimelockActive,
    
    // 指令相关错误（合并指令验证错误）
    #[msg("指令格式无效")]
    InvalidInstruction,
} 