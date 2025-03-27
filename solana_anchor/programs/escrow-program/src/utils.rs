use anchor_lang::prelude::*;
use crate::{error::*, state::{MAX_PAYMENT_TARGETS, EscrowAccount}, ed25519};

pub fn verify_payment_amounts<T>(
    payment_amounts: &[u64],
    escrow_account: &T,
) -> Result<()>
where
    T: AsRef<EscrowAccount>,
{
    let base = escrow_account.as_ref();
    
    require!(payment_amounts.len() <= MAX_PAYMENT_TARGETS, EscrowError::InvalidPaymentParameters);
    require!(payment_amounts.len() > 0, EscrowError::InvalidPaymentParameters);
    
    let total_amount: u64 = payment_amounts.iter().sum();
    require!(total_amount <= base.amount, EscrowError::InvalidPaymentParameters);
    
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
    
    // 构造消息
    let message = construct_message(&base.unique_id, recipients, payment_amounts);
    
    // 时间锁检查
    let time_expired = current_time >= base.unlock_time;
    
    // 获取授权签名者
    let buyer = base.buyer;
    let seller = base.seller;
    
    if !time_expired {
        // 时间锁未到期 - 验证签名
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
            EscrowError::SignatureVerificationFailed
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
            EscrowError::Unauthorized
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
    
    require!(current_index > 0, EscrowError::SignatureVerificationFailed);
    
    let prev_ix = solana_program::sysvar::instructions::load_instruction_at_checked(
        (current_index as usize) - 1,
        instructions_sysvar,
    )?;
    
    require!(
        prev_ix.program_id == solana_program::ed25519_program::ID,
        EscrowError::SignatureVerificationFailed
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
