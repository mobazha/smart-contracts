use anchor_lang::prelude::*;
use crate::{error::*, state::{MAX_PAYMENT_TARGETS, EscrowAccount}, ed25519};
use chrono::{TimeZone, Utc};

pub fn verify_payment_amounts(
    payment_amounts: &[u64],
    escrow_account: &EscrowAccount,
) -> Result<()> {
    require!(payment_amounts.len() <= MAX_PAYMENT_TARGETS, EscrowError::TooManyRecipients);
    require!(payment_amounts.len() > 0, EscrowError::InvalidPaymentParameters);
    
    for amount in payment_amounts {
        require!(*amount > 0, EscrowError::ZeroPaymentAmount);
    }
    
    let total_amount: u64 = payment_amounts.iter().sum();
    require!(total_amount <= escrow_account.amount, EscrowError::PaymentAmountExceedsEscrow);
    
    Ok(())
}

pub fn construct_message(unique_id: &[u8; 20], recipients: &[Option<Pubkey>], amounts: &[u64]) -> Vec<u8> {
    let mut message = Vec::new();
    message.extend_from_slice(unique_id);
    
    let len = std::cmp::min(amounts.len(), recipients.len());
    
    for i in 0..len {
        if recipients[i].is_none() {
            break;
        }
        
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

    let all_signers = verify_ed25519_instructions(
        instructions_sysvar,
        signatures,
        &message,
    )?;
    
    if !time_expired {
        // Check if this is a seller refund case
        let is_seller_refund = recipients.len() == 1 && 
                              recipients[0].is_some() && 
                              recipients[0].unwrap() == base.payer_address &&
                              all_signers.contains(&seller);
        
        if !is_seller_refund {
            // Filter valid signers
            let valid_signers: Vec<_> = all_signers.into_iter()
                .filter(|signer| {
                    *signer == buyer || 
                    *signer == seller || 
                    (base.moderator.is_some() && *signer == base.moderator.unwrap())
                })
                .collect();

            // Check the number of valid signatures
            require!(
                valid_signers.len() >= required_signatures as usize,
                EscrowError::InsufficientSignatures
            );
        }
    } else {
        // Timelock has expired - only seller signature is required
        require!(
            all_signers.contains(&seller),
            EscrowError::InvalidSigner
        );
    }
    
    Ok(())
}

pub fn verify_ed25519_instructions(
    instructions_sysvar: &AccountInfo,
    expected_signatures: &[Vec<u8>],
    expected_message: &[u8],
) -> Result<Vec<Pubkey>> {
    let mut valid_signers = Vec::new();
    
    // Get current instruction index and previous instruction
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
    
    verify_payment_amounts(payment_amounts, base)?;
    
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

/// Convert byte array to hexadecimal string
pub fn bytes_to_hex_string(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

/// Convert Unix timestamp to readable datetime string
pub fn format_timestamp(timestamp: i64) -> String {
    match Utc.timestamp_opt(timestamp, 0) {
        chrono::LocalResult::Single(dt) => {
            format!("{} UTC", dt.format("%Y-%m-%d %H:%M:%S"))
        },
        _ => format!("{} (invalid timestamp)", timestamp),
    }
}

pub fn verify_signatures_without_timelock(
    escrow_account: &EscrowAccount,
    signatures: &[Vec<u8>],
    payment_amounts: &[u64],
    recipients: &[Option<Pubkey>],
    instructions_sysvar: &AccountInfo,
) -> Result<()> {
    let message = construct_message(&escrow_account.unique_id, recipients, payment_amounts);
    
    let all_signers = verify_ed25519_instructions(
        instructions_sysvar,
        signatures,
        &message,
    )?;
    
    // Filter valid signers
    let valid_signers: Vec<_> = all_signers.into_iter()
        .filter(|signer| {
            *signer == escrow_account.buyer || 
            *signer == escrow_account.seller || 
            (escrow_account.moderator.is_some() && *signer == escrow_account.moderator.unwrap())
        })
        .collect();

    // Check the number of valid signatures
    require!(
        valid_signers.len() >= escrow_account.required_signatures as usize,
        EscrowError::InsufficientSignatures
    );
    
    Ok(())
}
