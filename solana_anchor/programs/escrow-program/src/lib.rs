use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod error;
pub mod utils;

use instructions::{initialize_sol::*, initialize_token::*, release_sol::*, release_token::*};

declare_id!("E8vcELiVSEk8BLyHGY697muumtfqm8t8vEP4Mt5thYg7");

#[program]
pub mod escrow_program {
    use super::*;

    // SOL托管指令
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

    // SPL代币托管指令
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
} 