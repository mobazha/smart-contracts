use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod error;
pub mod utils;

use instructions::{
    initialize_sol::*, 
    initialize_token::*, 
    release_sol::*, 
    release_token::*,
    initialize_pool::*,
    initialize_token_pool::*,
    initialize_sol_record::*,
    initialize_token_record::*,
    release_sol_record::*,
    release_token_record::*
};

declare_id!("E8vcELiVSEk8BLyHGY697muumtfqm8t8vEP4Mt5thYg7");

#[program]
pub mod escrow_program {
    use super::*;

    // SOL托管指令 - 原始模式
    pub fn initialize_sol(
        ctx: Context<InitializeSol>,
        moderator: Option<Pubkey>,
        unique_id: [u8; 20],
        required_signatures: u8,
        unlock_hours: u64,
        amount: u64,
    ) -> Result<()> {
        instructions::initialize_sol::handler(
            ctx,
            moderator,
            unique_id,
            required_signatures,
            unlock_hours,
            amount,
        )
    }

    pub fn release_sol(
        ctx: Context<ReleaseSol>,
        payment_amounts: Vec<u64>,
        signatures: Vec<Vec<u8>>
    ) -> Result<()> {
        instructions::release_sol::handler(ctx, payment_amounts, signatures)
    }

    // SPL代币托管指令 - 原始模式
    pub fn initialize_token(
        ctx: Context<InitializeToken>,
        moderator: Option<Pubkey>,
        unique_id: [u8; 20],
        required_signatures: u8,
        unlock_hours: u64,
        amount: u64,
    ) -> Result<()> {
        instructions::initialize_token::handler(
            ctx,
            moderator,
            unique_id,
            required_signatures,
            unlock_hours,
            amount,
        )
    }

    pub fn release_token(
        ctx: Context<ReleaseToken>,
        payment_amounts: Vec<u64>,
        signatures: Vec<Vec<u8>>
    ) -> Result<()> {
        instructions::release_token::handler(ctx, payment_amounts, signatures)
    }
    
    // 资金池初始化指令
    pub fn initialize_sol_pool(ctx: Context<InitializeSolPool>) -> Result<()> {
        instructions::initialize_pool::handler(ctx)
    }
    
    pub fn initialize_token_pool(ctx: Context<InitializeTokenPool>) -> Result<()> {
        instructions::initialize_token_pool::handler(ctx)
    }
    
    // SOL托管指令 - 记录账户模式
    pub fn initialize_sol_record(
        ctx: Context<InitializeSolRecord>,
        moderator: Option<Pubkey>,
        unique_id: [u8; 20],
        required_signatures: u8,
        unlock_hours: u64,
        amount: u64,
    ) -> Result<()> {
        instructions::initialize_sol_record::handler(
            ctx,
            moderator,
            unique_id,
            required_signatures,
            unlock_hours,
            amount,
        )
    }
    
    pub fn release_sol_record(
        ctx: Context<ReleaseSolRecord>,
        payment_amounts: Vec<u64>,
        signatures: Vec<Vec<u8>>
    ) -> Result<()> {
        instructions::release_sol_record::handler(ctx, payment_amounts, signatures)
    }
    
    // SPL代币托管指令 - 记录账户模式
    pub fn initialize_token_record(
        ctx: Context<InitializeTokenRecord>,
        moderator: Option<Pubkey>,
        unique_id: [u8; 20],
        required_signatures: u8,
        unlock_hours: u64,
        amount: u64,
    ) -> Result<()> {
        instructions::initialize_token_record::handler(
            ctx,
            moderator,
            unique_id,
            required_signatures,
            unlock_hours,
            amount,
        )
    }
    
    pub fn release_token_record(
        ctx: Context<ReleaseTokenRecord>,
        payment_amounts: Vec<u64>,
        signatures: Vec<Vec<u8>>
    ) -> Result<()> {
        instructions::release_token_record::handler(ctx, payment_amounts, signatures)
    }
} 