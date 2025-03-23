use solana_account_info::AccountInfo;
use solana_program_entrypoint::ProgramResult;
use solana_pubkey::Pubkey;

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

// 程序入口点
#[cfg(not(feature = "no-entrypoint"))]
solana_program_entrypoint::entrypoint!(process_instruction);

// 处理指令
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    processor::Processor::process(program_id, accounts, instruction_data)
} 