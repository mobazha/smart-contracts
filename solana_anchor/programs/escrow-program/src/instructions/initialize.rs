use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use crate::{state::*, error::*};

#[derive(Accounts)]
#[instruction(
    moderator: Option<Pubkey>,
    unique_id: [u8; 20],
    required_signatures: u8,
    unlock_hours: u64,
    token_type: TokenType
)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,
    
    #[account(
        init,
        payer = buyer,
        space = Escrow::LEN,
        seeds = [
            ESCROW_SEED_PREFIX,
            buyer.key().as_ref(),
            seller.key().as_ref(),
            moderator.as_ref().map_or(&[], |m| m.as_ref()),
            &unique_id
        ],
        bump
    )]
    pub escrow_account: Account<'info, Escrow>,
    
    /// CHECK: 卖家账户，只读取
    pub seller: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
    
    // 可选的SPL代币相关账户
    #[account(
        mut,
        owner = token::ID,
        constraint = match token_type { 
            TokenType::Spl(mint_key) => mint.key() == mint_key, 
            _ => true 
        }
    )]
    pub mint: Option<Account<'info, Mint>>,
    
    pub token_program: Option<Program<'info, Token>>,
    
    #[account(
        init_if_needed,
        payer = buyer,
        associated_token::mint = mint,
        associated_token::authority = escrow_account
    )]
    pub escrow_token_account: Option<Account<'info, TokenAccount>>,
}

pub fn handler(
    ctx: Context<Initialize>,
    moderator: Option<Pubkey>,
    unique_id: [u8; 20],
    required_signatures: u8,
    unlock_hours: u64,
    token_type: TokenType,
) -> Result<()> {
    // 验证参数
    require!(
        required_signatures <= MAX_REQUIRED_SIGNATURES,
        EscrowError::TooManyRequiredSignatures
    );
    
    // 计算解锁时间戳
    let unlock_time = if unlock_hours > 0 {
        ctx.accounts.clock.unix_timestamp
            .checked_add((unlock_hours as i64).checked_mul(3600).ok_or(error!(EscrowError::InvalidAccountData))?)
            .ok_or(error!(EscrowError::InvalidAccountData))?
    } else {
        0 // 不设置时间锁
    };
    
    // 初始化托管状态
    let escrow = &mut ctx.accounts.escrow_account;
    escrow.state = EscrowState::Active;
    escrow.buyer = ctx.accounts.buyer.key();
    escrow.seller = ctx.accounts.seller.key();
    escrow.moderator = moderator;
    escrow.token_type = token_type;
    escrow.amount = 0; // 将在 Deposit 时设置
    escrow.unlock_time = unlock_time;
    escrow.required_signatures = required_signatures;
    escrow.buyer_signed = false;
    escrow.seller_signed = false;
    escrow.moderator_signed = false;
    escrow.is_initialized = true;
    escrow.unique_id = unique_id;
    
    msg!("托管已初始化: {:?}", ctx.accounts.escrow_account.key());
    msg!("买家: {:?}", ctx.accounts.buyer.key());
    msg!("时间锁: {}", unlock_time);
    
    Ok(())
} 