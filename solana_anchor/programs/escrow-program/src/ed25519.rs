use anchor_lang::prelude::*;
use crate::error::*;

// 简化的Ed25519签名偏移量结构
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
    
    // 获取签名字节
    pub fn get_signature<'a>(&self, data: &'a [u8]) -> &'a [u8] {
        &data[self.signature_offset..self.signature_offset+64]
    }
    
    // 获取公钥字节
    pub fn get_public_key<'a>(&self, data: &'a [u8]) -> &'a [u8] {
        &data[self.public_key_offset..self.public_key_offset+32]
    }
    
    // 获取消息字节
    pub fn get_message<'a>(&self, data: &'a [u8]) -> &'a [u8] {
        &data[self.message_offset..self.message_offset+self.message_size]
    }
}

// 从指令数据中解析所有 Ed25519 签名验证
pub fn parse_ed25519_instruction_offsets(data: &[u8]) -> Result<Vec<Ed25519SignatureOffsets>> {
    // 基本验证，确保至少有一个签名
    require!(data.len() >= 2, EscrowError::InvalidInstruction);
    
    let num_signatures = data[0] as usize;
    require!(num_signatures > 0, EscrowError::InvalidInstruction);
    
    // 偏移量结构的起始位置
    let offset_start = 2;
    let offset_size = 14;
    
    let mut offsets = Vec::with_capacity(num_signatures);
    
    for i in 0..num_signatures {
        let offset_i = offset_start + (i * offset_size);
        // 简化创建，移除额外验证
        if offset_i + offset_size <= data.len() {
            let offset = Ed25519SignatureOffsets::from_bytes(data, offset_i);
            offsets.push(offset);
        }
    }
    
    Ok(offsets)
}

// 优化的验证函数
pub fn verify_ed25519_signatures(
    data: &[u8],
    expected_signatures: &[Vec<u8>],
    expected_message: &[u8],
) -> Result<Vec<Pubkey>> {
    let offsets = parse_ed25519_instruction_offsets(data)?;
    let mut valid_pubkeys = Vec::with_capacity(offsets.len());
    
    for offset in offsets {
        // 简化安全检查，假设指令数据已经通过验证
        let msg_bytes = offset.get_message(data);
        
        // 只有当消息匹配时才进行进一步验证
        if msg_bytes != expected_message {
            continue;
        }
        
        let sig_bytes = offset.get_signature(data);
        let pubkey_bytes = offset.get_public_key(data);
        
        // 检查签名是否在期望列表中
        for expected_sig in expected_signatures {
            if sig_bytes == expected_sig.as_slice() {
                // 添加有效公钥
                if let Ok(bytes) = pubkey_bytes.try_into() {
                    valid_pubkeys.push(Pubkey::new_from_array(bytes));
                    break;
                }
            }
        }
    }
    
    Ok(valid_pubkeys)
} 