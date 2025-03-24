use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("金额不能为零")]
    ZeroAmount,
    
    #[msg("金额超出范围")]
    AmountOverflow,
    
    #[msg("金额不足")]
    InsufficientFunds,
    
    #[msg("无效的账户数据")]
    InvalidAccountData,
    
    #[msg("无效的代币账户")]
    InvalidTokenAccount,
    
    #[msg("无效的签名者")]
    InvalidSigner,
    
    #[msg("账户已经被签名")]
    AlreadySigned,
    
    #[msg("未满足所需签名数量")]
    InsufficientSignatures,
    
    #[msg("支付目标无效")]
    InvalidPaymentTargets,
    
    #[msg("支付目标总额与托管金额不匹配")]
    PaymentAmountMismatch,
    
    #[msg("超出最大支付目标数量")]
    TooManyPaymentTargets,
    
    #[msg("所需签名数量超过最大值")]
    TooManyRequiredSignatures,
    
    #[msg("没有签名")]
    NoSignature,
} 