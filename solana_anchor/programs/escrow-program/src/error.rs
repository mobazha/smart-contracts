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
    
    // 新增更细粒度的错误类型
    #[msg("签名数量不足")]
    InsufficientSignatures,
    
    #[msg("签名者不是授权用户")]
    InvalidSigner,
    
    #[msg("托管金额不足")]
    InsufficientFunds,
    
    #[msg("接收方账户无效")]
    InvalidRecipient,
    
    #[msg("接收方数量超过最大限制")]
    TooManyRecipients,
    
    #[msg("总支付金额超过托管金额")]
    PaymentAmountExceedsEscrow,
    
    #[msg("支付金额不能为零")]
    ZeroPaymentAmount,
    
    #[msg("代币铸币地址不匹配")]
    TokenMintMismatch,
    
    #[msg("必需签名数量无效")]
    InvalidRequiredSignatures,
    
    #[msg("Ed25519指令未找到或无效")]
    InvalidEd25519Instruction,
} 