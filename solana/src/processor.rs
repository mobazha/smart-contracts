use borsh::BorshDeserialize;
use solana_account_info::{AccountInfo, next_account_info};
use solana_program_error::ProgramError;
use solana_pubkey::Pubkey;
use solana_program::{
    system_instruction,
    sysvar::{rent::Rent, Sysvar, clock::Clock},
    program::{invoke, invoke_signed},
    msg,
};
use solana_program_entrypoint::ProgramResult;
use spl_token::instruction as token_instruction;
use spl_token::state::Account;
use solana_program_pack::Pack;

use crate::{
    instruction::{EscrowInstruction, MAX_PAYMENT_TARGETS},
    state::{Escrow, EscrowState, TokenType, PaymentTarget, ESCROW_SEED_PREFIX},
    error::EscrowError,
};

pub struct Processor;

impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        let instruction = EscrowInstruction::try_from_slice(instruction_data)
            .map_err(|_| EscrowError::InvalidInstruction)?;

        match instruction {
            EscrowInstruction::Initialize {
                moderator,
                unique_id,
                required_signatures, 
                unlock_hours, 
                token_type,
            } => {
                Self::process_initialize(
                    program_id,
                    accounts,
                    moderator,
                    unique_id,
                    required_signatures,
                    unlock_hours,
                    token_type,
                )
            }
            EscrowInstruction::Deposit { amount } => {
                Self::process_deposit(accounts, amount)
            }
            EscrowInstruction::Sign => {
                Self::process_sign(accounts)
            }
            EscrowInstruction::Release { payment_targets } => {
                Self::process_release(program_id, accounts, payment_targets)
            }
        }
    }

    fn process_initialize(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        moderator: Option<Pubkey>,
        unique_id: [u8; 20],
        required_signatures: u8,
        unlock_hours: u64,
        token_type: TokenType,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let buyer = next_account_info(account_info_iter)?;
        let escrow_account = next_account_info(account_info_iter)?;
        let seller = next_account_info(account_info_iter)?;
        let system_program = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;
        let clock = &Clock::from_account_info(next_account_info(account_info_iter)?)?;

        // 验证签名
        if !buyer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        // 计算解锁时间戳（当unlock_hours为0时不设置时间锁）
        let unlock_time = if unlock_hours > 0 {
            clock.unix_timestamp.checked_add(
                (unlock_hours as i64).checked_mul(3600)
                    .ok_or(ProgramError::InvalidArgument)?
            ).ok_or(ProgramError::InvalidArgument)?
        } else {
            0 // 不设置时间锁
        };

        // 在 process_initialize 中，如果是 SPL token，需要验证 mint account
        let token_type_clone = token_type.clone();  // 克隆 token_type
        let mint_account = match &token_type {
            TokenType::Spl(mint) => {
                let mint_acc = next_account_info(account_info_iter)?;
                if *mint_acc.key != *mint {
                    return Err(ProgramError::InvalidAccountData);
                }
                Some(mint_acc)
            },
            _ => None,
        };

        // 初始化托管状态
        let escrow = Escrow {
            state: EscrowState::Active,
            buyer: *buyer.key,
            seller: *seller.key,
            moderator,
            token_type: token_type_clone,  // 使用克隆的值
            amount: 0, // 将在 Deposit 时设置
            unlock_time,  // 0表示没有时间锁
            required_signatures,
            buyer_signed: false,
            seller_signed: false,
            moderator_signed: false,
            is_initialized: true,
            unique_id,
        };

        // 在 process_initialize 中应该使用 PDA 而不是直接创建账户
        let moderator_ref = moderator.as_ref().map(|m| m.as_ref()).unwrap_or(&[]);
        let mut seeds = vec![
            ESCROW_SEED_PREFIX,
            buyer.key.as_ref(),
            seller.key.as_ref(),
            moderator_ref,
            &unique_id,
        ];

        let (_, bump_seed) = Pubkey::find_program_address(&seeds[..], program_id);
        let bump = [bump_seed];
        seeds.push(&bump);
        let signer_seeds = &seeds[..];

        invoke_signed(
            &system_instruction::create_account(
                buyer.key,
                escrow_account.key,
                rent.minimum_balance(Escrow::LEN),
                Escrow::LEN as u64,
                program_id,
            ),
            &[buyer.clone(), escrow_account.clone(), system_program.clone()],
            &[signer_seeds],
        )?;

        // 在 TokenType::Spl 分支中需要初始化代币账户
        if let TokenType::Spl(mint) = token_type {
            let token_program = next_account_info(account_info_iter)?;
            let escrow_token_account = next_account_info(account_info_iter)?;
            let rent_account = next_account_info(account_info_iter)?;  // 获取 rent 账户
            
            invoke_signed(
                &spl_token::instruction::initialize_account(
                    token_program.key,
                    escrow_token_account.key,
                    &mint,
                    escrow_account.key,
                )?,
                &[
                    escrow_token_account.clone(),
                    mint_account.unwrap().clone(),
                    escrow_account.clone(),
                    rent_account.clone(),  // 使用 rent 账户
                    token_program.clone(),
                ],
                &[&signer_seeds[..]],
            )?;
        }

        escrow.pack_into_slice(&mut escrow_account.data.borrow_mut());
        msg!("Escrow initialized: {:?}", escrow_account.key);
        msg!("Initializing escrow with buyer: {:?}", buyer.key);
        msg!("Time lock: {}", unlock_time);
        Ok(())
    }

    fn process_deposit(
        accounts: &[AccountInfo],
        amount: u64,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let depositor = next_account_info(account_info_iter)?;
        let escrow_account = next_account_info(account_info_iter)?;

        // 基本检查
        if !depositor.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut escrow = Escrow::unpack_from_slice(&escrow_account.data.borrow())?;
        if escrow.state != EscrowState::Active {
            return Err(EscrowError::AlreadyCompleted.into());
        }

        if amount == 0 {
            return Err(ProgramError::InvalidArgument);
        }

        match escrow.token_type {
            TokenType::Sol => {
                let system_program = next_account_info(account_info_iter)?;
                // 检查是否是系统程序
                if *system_program.key != solana_program::system_program::id() {
                    return Err(ProgramError::InvalidAccountData);
                }

                invoke(
                    &system_instruction::transfer(depositor.key, escrow_account.key, amount),
                    &[
                        depositor.clone(),
                        escrow_account.clone(),
                        system_program.clone(),
                    ],
                )?;
            },
            TokenType::Spl(mint) => {
                let token_program = next_account_info(account_info_iter)?;
                let source = next_account_info(account_info_iter)?;
                let destination = next_account_info(account_info_iter)?;

                // 检查代币程序ID
                if *token_program.key != spl_token::id() {
                    return Err(ProgramError::InvalidAccountData);
                }

                // 检查代币账户的mint是否匹配
                let source_account = Account::unpack(&source.data.borrow())?;
                if source_account.mint != mint {
                    return Err(ProgramError::InvalidAccountData);
                }

                invoke(
                    &token_instruction::transfer(
                        token_program.key,
                        source.key,
                        destination.key,
                        depositor.key,
                        &[],
                        amount,
                    )?,
                    &[
                        source.clone(),
                        destination.clone(),
                        depositor.clone(),
                        token_program.clone(),
                    ],
                )?;
            }
        }

        escrow.amount = escrow.amount.checked_add(amount)
            .ok_or(ProgramError::InvalidArgument)?;
        escrow.pack_into_slice(&mut escrow_account.data.borrow_mut());
        msg!("Amount deposited: {}", amount);
        Ok(())
    }

    fn process_sign(accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let signer = next_account_info(account_info_iter)?;
        let escrow_account = next_account_info(account_info_iter)?;

        if !signer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mut escrow = Escrow::unpack_from_slice(&escrow_account.data.borrow())?;

        if escrow.state != EscrowState::Active {
            return Err(EscrowError::AlreadyCompleted.into());
        }

        // 更新对应角色的签名状态
        if *signer.key == escrow.buyer {
            escrow.buyer_signed = true;
        } else if *signer.key == escrow.seller {
            escrow.seller_signed = true;
        } else if let Some(moderator) = escrow.moderator {
            if *signer.key == moderator {
                escrow.moderator_signed = true;
            } else {
                return Err(EscrowError::InvalidSigner.into());
            }
        } else {
            return Err(EscrowError::InvalidSigner.into());
        }

        escrow.pack_into_slice(&mut escrow_account.data.borrow_mut());
        Ok(())
    }

    fn process_release(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        payment_targets: Vec<PaymentTarget>,
    ) -> ProgramResult {
        // 验证付款目标数量
        if payment_targets.len() > MAX_PAYMENT_TARGETS {
            return Err(ProgramError::InvalidArgument);
        }

        let account_info_iter = &mut accounts.iter();
        let initiator = next_account_info(account_info_iter)?;
        let escrow_account = next_account_info(account_info_iter)?;
        let clock = &Clock::from_account_info(next_account_info(account_info_iter)?)?;
        let mut escrow = Escrow::unpack_from_slice(&escrow_account.data.borrow())?;

        // 验证状态
        if escrow.state != EscrowState::Active {
            return Err(EscrowError::AlreadyCompleted.into());
        }

        // 验证 initiator 的身份
        if *initiator.key != escrow.buyer && *initiator.key != escrow.seller {
            return Err(ProgramError::InvalidAccountData);
        }

        // 检查签名数量
        let signed_count = escrow.buyer_signed as u8 
            + escrow.seller_signed as u8 
            + (escrow.moderator.is_some() && escrow.moderator_signed) as u8;

        // 检查签名数量或时间锁
        let time_lock_passed = escrow.unlock_time > 0 && 
            clock.unix_timestamp >= escrow.unlock_time;

        if signed_count < escrow.required_signatures {
            // 只有设置了时间锁（unlock_time > 0）时才检查
            if !time_lock_passed {
                return Err(EscrowError::InsufficientSignatures.into());
            }
            // 超过时间锁后，只需要卖家签名
            if !escrow.seller_signed {
                return Err(EscrowError::InsufficientSignatures.into());
            }
        }

        // 在 process_release 中需要添加总金额验证
        let total_amount: u64 = payment_targets.iter()
            .map(|target| target.amount)
            .sum();
        if total_amount > escrow.amount {
            return Err(ProgramError::InvalidArgument);
        }

        match escrow.token_type {
            TokenType::Sol => {
                for target in payment_targets.iter() {
                    let recipient = next_account_info(account_info_iter)?;
                    if target.recipient != *recipient.key {
                        return Err(ProgramError::InvalidAccountData);
                    }
                    **escrow_account.try_borrow_mut_lamports()? -= target.amount;
                    **recipient.try_borrow_mut_lamports()? += target.amount;
                }

                // 关闭账户：将剩余的租金返还给买家
                let buyer = next_account_info(account_info_iter)?;
                let remaining_lamports = escrow_account.lamports();
                **escrow_account.try_borrow_mut_lamports()? = 0;
                **buyer.try_borrow_mut_lamports()? += remaining_lamports;
            },
            TokenType::Spl(_) => {
                let token_program = next_account_info(account_info_iter)?;
                let escrow_token_account = next_account_info(account_info_iter)?;
                let buyer = next_account_info(account_info_iter)?;

                let moderator_ref = escrow.moderator.as_ref().map(|m| m.as_ref()).unwrap_or(&[]);
                let mut seeds = vec![
                    ESCROW_SEED_PREFIX,
                    escrow.buyer.as_ref(),
                    escrow.seller.as_ref(),
                    moderator_ref,
                    &escrow.unique_id,
                ];

                let (_, bump_seed) = Pubkey::find_program_address(&seeds[..], program_id);
                let bump = [bump_seed];
                seeds.push(&bump);
                let signer_seeds = &seeds[..];

                for target in payment_targets.iter() {
                    let recipient_token_account = next_account_info(account_info_iter)?;
                    
                    invoke_signed(
                        &token_instruction::transfer(
                            token_program.key,
                            escrow_token_account.key,
                            recipient_token_account.key,
                            escrow_account.key,
                            &[],
                            target.amount,
                        )?,
                        &[
                            escrow_token_account.clone(),
                            recipient_token_account.clone(),
                            escrow_account.clone(),
                            token_program.clone(),
                        ],
                        &[signer_seeds],
                    )?;
                }

                // 关闭代币账户，返还租金
                invoke_signed(
                    &token_instruction::close_account(
                        token_program.key,
                        escrow_token_account.key,
                        &escrow.buyer,
                        escrow_account.key,
                        &[],
                    )?,
                    &[
                        escrow_token_account.clone(),
                        buyer.clone(),
                        escrow_account.clone(),
                        token_program.clone(),
                    ],
                    &[signer_seeds],
                )?;
            }
        }

        // 清除账户数据
        escrow.state = EscrowState::Completed;
        escrow.pack_into_slice(&mut escrow_account.data.borrow_mut());
        msg!("Release initiated by: {:?}", initiator.key);
        msg!("Time lock: {}", escrow.unlock_time);
        Ok(())
    }
}
