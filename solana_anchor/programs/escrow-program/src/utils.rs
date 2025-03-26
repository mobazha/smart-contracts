use anchor_lang::prelude::*;
use solana_program::ed25519_program::ID as ed25519_program_id;
use solana_program::instruction::Instruction;
use crate::{error::*, state::*};

pub trait EscrowAccount {
    fn buyer(&self) -> &Pubkey;
    fn seller(&self) -> &Pubkey;
    fn moderator(&self) -> Option<Pubkey>;
    fn required_signatures(&self) -> u8;
    fn unique_id(&self) -> &[u8; 20];
    fn amount(&self) -> u64;
    fn unlock_time(&self) -> i64;
}

impl EscrowAccount for TokenEscrow {
    fn buyer(&self) -> &Pubkey { &self.buyer }
    fn seller(&self) -> &Pubkey { &self.seller }
    fn moderator(&self) -> Option<Pubkey> { self.moderator }
    fn required_signatures(&self) -> u8 { self.required_signatures }
    fn unique_id(&self) -> &[u8; 20] { &self.unique_id }
    fn amount(&self) -> u64 { self.amount }
    fn unlock_time(&self) -> i64 { self.unlock_time }
}

impl EscrowAccount for SolEscrow {
    fn buyer(&self) -> &Pubkey { &self.buyer }
    fn seller(&self) -> &Pubkey { &self.seller }
    fn moderator(&self) -> Option<Pubkey> { self.moderator }
    fn required_signatures(&self) -> u8 { self.required_signatures }
    fn unique_id(&self) -> &[u8; 20] { &self.unique_id }
    fn amount(&self) -> u64 { self.amount }
    fn unlock_time(&self) -> i64 { self.unlock_time }
}

impl EscrowAccount for SolEscrowRecord {
    fn buyer(&self) -> &Pubkey { &self.buyer }
    fn seller(&self) -> &Pubkey { &self.seller }
    fn moderator(&self) -> Option<Pubkey> { self.moderator }
    fn required_signatures(&self) -> u8 { self.required_signatures }
    fn unique_id(&self) -> &[u8; 20] { &self.unique_id }
    fn amount(&self) -> u64 { self.amount }
    fn unlock_time(&self) -> i64 { self.unlock_time }
}

impl EscrowAccount for TokenEscrowRecord {
    fn buyer(&self) -> &Pubkey { &self.buyer }
    fn seller(&self) -> &Pubkey { &self.seller }
    fn moderator(&self) -> Option<Pubkey> { self.moderator }
    fn required_signatures(&self) -> u8 { self.required_signatures }
    fn unique_id(&self) -> &[u8; 20] { &self.unique_id }
    fn amount(&self) -> u64 { self.amount }
    fn unlock_time(&self) -> i64 { self.unlock_time }
}

pub fn verify_payment_amounts<T: EscrowAccount + AccountSerialize + AccountDeserialize + Clone>(
    payment_amounts: &[u64],
    escrow_account: &Account<T>,
) -> Result<()> {
    require!(payment_amounts.len() <= MAX_PAYMENT_TARGETS, EscrowError::TooManyPaymentTargets);
    require!(payment_amounts.len() > 0, EscrowError::InvalidPaymentTargets);
    
    let total_amount: u64 = payment_amounts.iter().sum();
    require!(total_amount <= escrow_account.amount(), EscrowError::PaymentAmountMismatch);
    
    Ok(())
}

pub fn verify_ed25519_signature(
    message: &[u8], 
    signature: &[u8],
    signer: &Pubkey,  // 添加签名者的公钥参数
) -> Result<()> {
    let ix = Instruction::new_with_bytes(
        ed25519_program_id,
        &[
            // ed25519 instruction format:
            // [public_key (32 bytes), signature (64 bytes), message_length (4 bytes), message (variable)]
            &signer.to_bytes(),  // 使用传入的签名者公钥
            signature, 
            &(message.len() as u32).to_le_bytes(),
            message
        ].concat(),
        vec![],  // ed25519 program doesn't need account metas
    );
    
    let result = solana_program::program::invoke(
        &ix,
        &[]  // ed25519 program doesn't need accounts
    );

    match result {
        Ok(_) => Ok(()),
        Err(_) => err!(EscrowError::InvalidSignature),
    }
}

// 构造消息的逻辑也可以抽取
pub fn construct_message(unique_id: &[u8; 20], amounts: &[u64]) -> Vec<u8> {
    let mut message = Vec::new();
    message.extend_from_slice(unique_id);
    for amount in amounts {
        message.extend_from_slice(&amount.to_le_bytes());
    }
    message
}

pub fn verify_release_signatures<T>(
    escrow_account: &T,
    signatures: &[Vec<u8>],
    payment_amounts: &[u64],
) -> Result<()>
where
    T: EscrowAccount + AccountDeserialize + AccountSerialize + Clone,
{
    let mut valid_signatures = 0u8;
    let message = construct_message(escrow_account.unique_id(), payment_amounts);
    
    for signature in signatures {
        // 尝试验证买家的签名
        if verify_ed25519_signature(&message, signature, escrow_account.buyer()).is_ok() {
            valid_signatures += 1;
            continue;
        }
        
        // 尝试验证卖家的签名
        if verify_ed25519_signature(&message, signature, escrow_account.seller()).is_ok() {
            valid_signatures += 1;
            continue;
        }
        
        // 如果有仲裁者，尝试验证仲裁者的签名
        if let Some(moderator) = escrow_account.moderator() {
            if verify_ed25519_signature(&message, signature, &moderator).is_ok() {
                valid_signatures += 1;
            }
        }
    }

    require!(
        valid_signatures >= escrow_account.required_signatures(),
        EscrowError::InsufficientSignatures
    );
    
    Ok(())
}

pub fn verify_signatures_with_timelock<T>(
    escrow_account: &T,
    signatures: &[Vec<u8>],
    payment_amounts: &[u64],
    current_time: i64,
) -> Result<()>
where
    T: EscrowAccount + AccountDeserialize + AccountSerialize + Clone,
{
    // 检查时间锁是否过期
    let time_expired = current_time >= escrow_account.unlock_time();
    
    if time_expired {
        msg!("Timelock expired. Only seller signature required.");
        // 时间锁过期，但卖家必须签名
        let message = construct_message(escrow_account.unique_id(), payment_amounts);
        let mut seller_signed = false;
        
        for signature in signatures {
            if verify_ed25519_signature(&message, signature, escrow_account.seller()).is_ok() {
                seller_signed = true;
                msg!("Seller signature verified successfully.");
                break;
            }
        }
        
        require!(seller_signed, EscrowError::InsufficientSignatures);
    } else {
        msg!("Timelock active. Verifying all required signatures.");
        verify_release_signatures(escrow_account, signatures, payment_amounts)?;
    }
    
    Ok(())
}

pub fn close_escrow_and_return_rent<'info>(
    escrow_account: &AccountInfo<'info>,
    buyer: &AccountInfo<'info>,
) -> Result<()> {
    let rent_lamports = escrow_account.lamports();
    **escrow_account.try_borrow_mut_lamports()? = 0;
    **buyer.try_borrow_mut_lamports()? += rent_lamports;
    Ok(())
}

// 工具函数：取消托管交易
pub fn cancel_transaction<T: AccountSerialize + AccountDeserialize + Clone>(
    escrow_record: &mut Account<T>,
    status: &mut TransactionStatus,
    current_time: i64,
) -> Result<()> {
    require!(*status == TransactionStatus::Pending, EscrowError::InvalidTransactionStatus);
    
    *status = TransactionStatus::Cancelled;
    
    // 使用更简单的方式完成 - 不直接操作内存
    // 注意：此处只是示例，如需精确修改特定字段应使用专用的指令
    msg!("交易已取消，时间：{}", current_time);
    
    Ok(())
}
