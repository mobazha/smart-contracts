use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod error;

use state::*;
use instructions::{initialize::*, release::*};

declare_id!("E8vcELiVSEk8BLyHGY697muumtfqm8t8vEP4Mt5thYg7");

#[program]
pub mod escrow_program {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        moderator: Option<Pubkey>,
        unique_id: [u8; 20],
        required_signatures: u8,
        unlock_hours: u64,
        token_type: TokenType,
        amount: u64,
    ) -> Result<()> {
        instructions::initialize::handler(
            ctx,
            moderator,
            unique_id,
            required_signatures,
            unlock_hours,
            token_type,
            amount,
        )
    }

    pub fn release(
        ctx: Context<Release>,
        payment_amounts: Vec<u64>,
        signatures: Vec<Vec<u8>>
    ) -> Result<()> {
        instructions::release::handler(ctx, payment_amounts, signatures)
    }
} 