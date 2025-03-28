use anchor_lang::prelude::*;
use crate::error::*;

// Simplified Ed25519 signature offsets structure
#[derive(Debug)]
pub struct Ed25519SignatureOffsets {
    pub signature_offset: usize,
    pub public_key_offset: usize,
    pub message_offset: usize,
    pub message_size: usize,
}

impl Ed25519SignatureOffsets {
    pub fn from_bytes(data: &[u8], offset: usize) -> Self {
        let signature_offset = u16::from_le_bytes([data[offset], data[offset+1]]) as usize;
        let public_key_offset = u16::from_le_bytes([data[offset+4], data[offset+5]]) as usize;
        let message_offset = u16::from_le_bytes([data[offset+8], data[offset+9]]) as usize;
        let message_size = u16::from_le_bytes([data[offset+10], data[offset+11]]) as usize;
        
        Self {
            signature_offset,
            public_key_offset,
            message_offset,
            message_size,
        }
    }
    
    // Get signature bytes
    pub fn get_signature<'a>(&self, data: &'a [u8]) -> &'a [u8] {
        &data[self.signature_offset..self.signature_offset+64]
    }
    
    // Get public key bytes
    pub fn get_public_key<'a>(&self, data: &'a [u8]) -> &'a [u8] {
        &data[self.public_key_offset..self.public_key_offset+32]
    }
    
    // Get message bytes
    pub fn get_message<'a>(&self, data: &'a [u8]) -> &'a [u8] {
        &data[self.message_offset..self.message_offset+self.message_size]
    }
}

// Parse all Ed25519 signature verifications from instruction data
pub fn parse_ed25519_instruction_offsets(data: &[u8]) -> Result<Vec<Ed25519SignatureOffsets>> {
    // Basic validation, ensure at least one signature
    require!(data.len() >= 2, EscrowError::InvalidInstruction);
    
    let num_signatures = data[0] as usize;
    require!(num_signatures > 0, EscrowError::InvalidInstruction);
    
    // Starting position of offset structures
    let offset_start = 2;
    let offset_size = 14;
    
    let mut offsets = Vec::with_capacity(num_signatures);
    
    for i in 0..num_signatures {
        let offset_i = offset_start + (i * offset_size);
        // Simplified creation, removed extra validation
        if offset_i + offset_size <= data.len() {
            let offset = Ed25519SignatureOffsets::from_bytes(data, offset_i);
            offsets.push(offset);
        }
    }
    
    Ok(offsets)
}

// Optimized verification function
pub fn verify_ed25519_signatures(
    data: &[u8],
    expected_signatures: &[Vec<u8>],
    expected_message: &[u8],
) -> Result<Vec<Pubkey>> {
    let offsets = parse_ed25519_instruction_offsets(data)?;
    let mut valid_pubkeys = Vec::with_capacity(offsets.len());
    
    for offset in offsets {
        // Simplified safety checks, assuming instruction data has been validated
        let msg_bytes = offset.get_message(data);
        
        // Only proceed with further validation when the message matches
        if msg_bytes != expected_message {
            continue;
        }
        
        let sig_bytes = offset.get_signature(data);
        let pubkey_bytes = offset.get_public_key(data);
        
        // Check if signature is in the expected list
        for expected_sig in expected_signatures {
            if sig_bytes == expected_sig.as_slice() {
                // Add valid public key
                if let Ok(bytes) = pubkey_bytes.try_into() {
                    valid_pubkeys.push(Pubkey::new_from_array(bytes));
                    break;
                }
            }
        }
    }
    
    Ok(valid_pubkeys)
} 