use anchor_lang::prelude::*;
use crate::{error::*, state::{MAX_PAYMENT_TARGETS, EscrowAccount}, ed25519};
use chrono::{TimeZone, Utc};

pub fn verify_payment_amounts<T>(
    payment_amounts: &[u64],
    escrow_account: &T,
) -> Result<()>
where
    T: AsRef<EscrowAccount>,
{
    let base = escrow_account.as_ref();
    
    require!(payment_amounts.len() <= MAX_PAYMENT_TARGETS, EscrowError::TooManyRecipients);
    require!(payment_amounts.len() > 0, EscrowError::InvalidPaymentParameters);
    
    for amount in payment_amounts {
        require!(*amount > 0, EscrowError::ZeroPaymentAmount);
    }
    
    let total_amount: u64 = payment_amounts.iter().sum();
    require!(total_amount <= base.amount, EscrowError::PaymentAmountExceedsEscrow);
    
    Ok(())
}

// 构造消息的逻辑也可以抽取
pub fn construct_message(unique_id: &[u8; 20], recipients: &[Option<Pubkey>], amounts: &[u64]) -> Vec<u8> {
    let mut message = Vec::new();
    message.extend_from_slice(unique_id);
    
    // 确保两个数组长度一致
    let len = std::cmp::min(amounts.len(), recipients.len());
    
    for i in 0..len {
        // 如果接收方为空，停止添加数据
        if recipients[i].is_none() {
            break;
        }
        
        // 先添加接收方，再添加金额
        message.extend_from_slice(recipients[i].as_ref().unwrap().as_ref());
        message.extend_from_slice(&amounts[i].to_le_bytes());
    }
    
    message
}

pub fn verify_signatures_with_timelock<T>(
    escrow_account: &T,
    signatures: &[Vec<u8>],
    payment_amounts: &[u64],
    recipients: &[Option<Pubkey>],
    current_time: i64,
    instructions_sysvar: &AccountInfo,
    required_signatures: u8,
) -> Result<()>
where
    T: AsRef<EscrowAccount>,
{
    let base = escrow_account.as_ref();
    
    let message = construct_message(&base.unique_id, recipients, payment_amounts);
    
    let time_expired = current_time >= base.unlock_time;
    
    let buyer = base.buyer;
    let seller = base.seller;
    
    if !time_expired {
        let all_signers = verify_ed25519_instructions(
            instructions_sysvar,
            signatures,
            &message,
        )?;
     
        // 过滤有效签名者
        let valid_signers: Vec<_> = all_signers.into_iter()
            .filter(|signer| {
                *signer == buyer || 
                *signer == seller || 
                (base.moderator.is_some() && *signer == base.moderator.unwrap())
            })
            .collect();

        // 检查有效签名数量
        require!(
            valid_signers.len() >= required_signatures as usize,
            EscrowError::InsufficientSignatures
        );
    } else {
        // 时间锁已过期 - 只需卖家签名
        let all_signers = verify_ed25519_instructions(
            instructions_sysvar,
            signatures,
            &message,
        )?;
        
        require!(
            all_signers.contains(&seller),
            EscrowError::InvalidSigner
        );
    }
    
    Ok(())
}

// 优化验证函数，传递所需签名数量
pub fn verify_ed25519_instructions(
    instructions_sysvar: &AccountInfo,
    expected_signatures: &[Vec<u8>],
    expected_message: &[u8],
) -> Result<Vec<Pubkey>> {
    let mut valid_signers = Vec::new();
    
    // 获取当前指令索引与前一个指令
    let current_index = solana_program::sysvar::instructions::load_current_index_checked(
        instructions_sysvar
    )?;
    
    require!(current_index > 0, EscrowError::InvalidEd25519Instruction);
    
    let prev_ix = solana_program::sysvar::instructions::load_instruction_at_checked(
        (current_index as usize) - 1,
        instructions_sysvar,
    )?;
    
    require!(
        prev_ix.program_id == solana_program::ed25519_program::ID,
        EscrowError::InvalidEd25519Instruction
    );

    let pubkeys = ed25519::verify_ed25519_signatures(
        &prev_ix.data, 
        expected_signatures, 
        expected_message,
    )?;
    
    valid_signers.extend(pubkeys);
    
    require!(!valid_signers.is_empty(), EscrowError::SignatureVerificationFailed);
    
    Ok(valid_signers)
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

pub fn process_release<T>(
    escrow_account: &T,
    signatures: &[Vec<u8>],
    payment_amounts: &[u64],
    recipients: &[Option<Pubkey>],
    current_time: i64,
    instructions_sysvar: &AccountInfo,
    transfer_function: impl FnOnce() -> Result<()>,
) -> Result<()> 
where 
    T: AsRef<EscrowAccount>,
{
    let base = escrow_account.as_ref();
    
    verify_payment_amounts(payment_amounts, escrow_account)?;
    
    verify_signatures_with_timelock(
        escrow_account,
        signatures,
        payment_amounts,
        recipients,
        current_time,
        instructions_sysvar,
        base.required_signatures,
    )?;
    
    transfer_function()
}

/// 将字节数组转换为十六进制字符串
pub fn bytes_to_hex_string(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

/// 将 Unix 时间戳转换为可读的日期时间字符串
pub fn format_timestamp(timestamp: i64) -> String {
    match Utc.timestamp_opt(timestamp, 0) {
        chrono::LocalResult::Single(dt) => {
            format!("{} UTC", dt.format("%Y-%m-%d %H:%M:%S"))
        },
        _ => format!("{} (invalid timestamp)", timestamp),
    }
}
