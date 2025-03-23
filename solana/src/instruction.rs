use borsh::{BorshDeserialize, BorshSerialize};
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use solana_program::{system_program, sysvar};
use crate::state::{PaymentTarget, TokenType};

pub const MAX_PAYMENT_TARGETS: usize = 4;
pub const MAX_REQUIRED_SIGNATURES: u8 = 2;

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub enum EscrowInstruction {
    /// 初始化托管
    Initialize {
        moderator: Option<Pubkey>,
        unique_id: [u8; 20],
        required_signatures: u8,
        unlock_hours: u64,
        token_type: TokenType,
    },

    /// 存入资金
    Deposit {
        amount: u64,
    },

    /// 签名
    Sign,

    /// 释放资金（需要足够的签名）
    Release {
        payment_targets: Vec<PaymentTarget>,
    },
}

pub fn initialize(
    program_id: &Pubkey,
    buyer: &Pubkey,
    escrow_account: &Pubkey,
    seller: &Pubkey,
    moderator: Option<&Pubkey>,
    unlock_hours: u64,
    required_signatures: u8,
    token_type: TokenType,
    unique_id: [u8; 20],
) -> Instruction {
    // 验证参数
    assert!(required_signatures <= MAX_REQUIRED_SIGNATURES, "Too many required signatures");
    
    let mut accounts = vec![
        AccountMeta::new(*buyer, true),
        AccountMeta::new(*escrow_account, false),
        AccountMeta::new_readonly(*seller, false),
        AccountMeta::new_readonly(system_program::id(), false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
        AccountMeta::new_readonly(sysvar::clock::id(), false),
    ];

    if let Some(moderator_pubkey) = moderator {
        accounts.push(AccountMeta::new_readonly(*moderator_pubkey, false));
    }

    Instruction {
        program_id: *program_id,
        accounts,
        data: EscrowInstruction::Initialize {
            unlock_hours,
            required_signatures,
            moderator: moderator.cloned(),
            token_type,
            unique_id,  // 使用传入的唯一ID
        }
        .try_to_vec()
        .unwrap(),
    }
}

pub fn release(
    program_id: &Pubkey,
    initiator: &Pubkey,
    escrow_account: &Pubkey,
    payment_targets: Vec<PaymentTarget>,
) -> Instruction {
    assert!(payment_targets.len() <= MAX_PAYMENT_TARGETS, "Too many payment targets");

    // 构建账户元数据列表
    let mut accounts = vec![
        AccountMeta::new_readonly(*initiator, true),      // 发起者，需要签名
        AccountMeta::new(*escrow_account, false),         // 托管账户，可写
        AccountMeta::new_readonly(sysvar::clock::id(), false), // 时钟账户
    ];

    // 添加所有付款目标的接收账户
    for target in payment_targets.iter() {
        accounts.push(AccountMeta::new(target.recipient, false)); // 接收账户，可写
    }

    Instruction {
        program_id: *program_id,
        accounts,
        data: EscrowInstruction::Release {
            payment_targets,
        }
        .try_to_vec()
        .unwrap(),
    }
}

pub fn deposit(
    program_id: &Pubkey,
    depositor: &Pubkey,
    escrow_account: &Pubkey,
    amount: u64,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new(*depositor, true),         // 存款人，需要签名
        AccountMeta::new(*escrow_account, false),   // 托管账户，可写
        AccountMeta::new_readonly(system_program::id(), false),
    ];

    Instruction {
        program_id: *program_id,
        accounts,
        data: EscrowInstruction::Deposit { amount }
            .try_to_vec()
            .unwrap(),
    }
}

pub fn sign(
    program_id: &Pubkey,
    signer: &Pubkey,
    escrow_account: &Pubkey,
) -> Instruction {
    let accounts = vec![
        AccountMeta::new_readonly(*signer, true),        // 签名者，需要签名
        AccountMeta::new(*escrow_account, false),        // 托管账户，可写
    ];

    Instruction {
        program_id: *program_id,
        accounts,
        data: EscrowInstruction::Sign
            .try_to_vec()
            .unwrap(),
    }
}
