use anchor_lang::prelude::*;

pub mod state;
pub mod instructions;
pub mod error;

use state::*;
use instructions::{initialize::*, deposit::*, sign::*, release::*};

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

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
    ) -> Result<()> {
        instructions::initialize::handler(
            ctx,
            moderator,
            unique_id,
            required_signatures,
            unlock_hours,
            token_type,
        )
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit::handler(ctx, amount)
    }

    pub fn sign(ctx: Context<Sign>) -> Result<()> {
        instructions::sign::handler(ctx)
    }

    pub fn release(
        ctx: Context<Release>,
        payment_amounts: Vec<u64>,
    ) -> Result<()> {
        instructions::release::handler(ctx, payment_amounts)
    }
} 