use anchor_lang::prelude::*;
use anchor_spl::token::{Token, Mint};
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
    
    // 根据token_type判断是否需要这些账户
    pub token_program: Option<Program<'info, Token>>,
    
    #[account(mut)]
    pub mint: Option<Account<'info, Mint>>,
    
    /// CHECK: 将在handler中验证
    #[account(mut)]
    pub escrow_token_account: Option<AccountInfo<'info>>,
    
    pub associated_token_program: Option<Program<'info, anchor_spl::associated_token::AssociatedToken>>,
}

pub fn handler(
    ctx: Context<Initialize>,
    moderator: Option<Pubkey>,
    unique_id: [u8; 20],
    required_signatures: u8,
    unlock_hours: u64, 
    token_type: TokenType,
) -> Result<()> {
    // 首先验证SPL代币账户,在token_type被移动前使用
    if let TokenType::Spl(mint_key) = &token_type {
        // 验证mint账户
        let mint = ctx.accounts.mint.as_ref()
            .ok_or(error!(EscrowError::InvalidTokenAccount))?;
        require!(mint.key() == *mint_key, EscrowError::InvalidMint);
        // ...代币账户初始化代码...
    }
    
    // 检查参数有效性
    require!(required_signatures > 0 && required_signatures <= MAX_REQUIRED_SIGNATURES, 
        EscrowError::TooManyRequiredSignatures);
    
    // 获取当前时间戳
    let current_time = ctx.accounts.clock.unix_timestamp;
    
    // 计算解锁时间戳
    let unlock_time = current_time + (unlock_hours as i64) * 3600;
    
    // 初始化托管账户数据
    let escrow = &mut ctx.accounts.escrow_account;
    escrow.buyer = ctx.accounts.buyer.key();
    escrow.seller = ctx.accounts.seller.key();
    escrow.moderator = moderator;
    escrow.amount = 0;
    escrow.unlock_time = unlock_time;
    escrow.required_signatures = required_signatures;
    escrow.buyer_signed = false;
    escrow.seller_signed = false;
    escrow.moderator_signed = false;
    escrow.state = EscrowState::Active;
    escrow.is_initialized = true;
    escrow.unique_id = unique_id;
    
    msg!("托管账户已初始化");
    msg!("买家: {}", escrow.buyer.to_string());
    msg!("卖家: {}", escrow.seller.to_string());
    if let Some(moderator) = escrow.moderator {
        msg!("仲裁人: {}", moderator.to_string());
    }
    msg!("解锁时间: {}", escrow.unlock_time);
    msg!("所需签名数: {}", escrow.required_signatures);
    
    // 这里会移动token_type
    escrow.token_type = token_type;
    
    Ok(())
} 