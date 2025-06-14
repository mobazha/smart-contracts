use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod error;
pub mod ed25519;
pub mod utils;

use instructions::{initialize_sol::*, initialize_token::*, release_sol::*, release_token::*};

declare_id!("25ecY9sGUkFyy78aYaSbdWGMgySSKZvPjQunf6Uk23qk");

#[program]
pub mod escrow_program {
    use super::*;

    // SOL initialize instruction
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

    // SPL token initialize instruction
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

    pub fn release_token_after_timeout(
        ctx: Context<ReleaseTokenAfterTimeout>,
        payment_amounts: Vec<u64>,
        signatures: Vec<Vec<u8>>
    ) -> Result<()> {
        instructions::release_token::handler_after_timeout(ctx, payment_amounts, signatures)
    }
} 