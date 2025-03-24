use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, CloseAccount, Transfer};
use crate::{state::*, error::*};
use spl_token::state::Account as SplTokenAccount;
use spl_token::solana_program::program_pack::Pack;

#[derive(Accounts)]
#[instruction(payment_amounts: Vec<u64>)]
pub struct Release<'info> {
    #[account(
        constraint = (initiator.key() == escrow_account.buyer || 
                     initiator.key() == escrow_account.seller || 
                     (escrow_account.moderator.is_some() && 
                      initiator.key() == escrow_account.moderator.unwrap())) 
                      @ EscrowError::InvalidSigner
    )]
    pub initiator: Signer<'info>,
    
    #[account(
        mut,
        constraint = escrow_account.state == EscrowState::Active @ EscrowError::AlreadyCompleted,
        constraint = escrow_account.is_initialized @ EscrowError::InvalidAccountData,
        seeds = [
            ESCROW_SEED_PREFIX,
            escrow_account.buyer.as_ref(),
            escrow_account.seller.as_ref(),
            escrow_account.moderator.as_ref().map_or(&[], |m| m.as_ref()),
            &escrow_account.unique_id
        ],
        bump
    )]
    pub escrow_account: Account<'info, Escrow>,
    
    pub clock: Sysvar<'info, Clock>,
    pub system_program: Program<'info, System>,
    
    // SOL支付或SPL代币支付需要的额外账户
    pub token_program: Option<Program<'info, Token>>,
    
    #[account(
        mut,
        token::authority = escrow_account
    )]
    pub escrow_token_account: Option<Account<'info, TokenAccount>>,
    
    /// CHECK: 买家账户，用于接收SPL代币账户的租金
    #[account(
        mut,
        constraint = buyer.key() == escrow_account.buyer
    )]
    pub buyer: AccountInfo<'info>,
    
    // 接收方账户需要在指令执行时动态检查，因为数量不固定
    /// CHECK: 接收方账户会在指令中验证
    #[account(mut)]
    pub recipients: UncheckedAccount<'info>,
    
    // 额外的接收方账户，最多支持4个接收方
    /// CHECK: 第二个接收方账户，如果有的话
    #[account(mut)]
    pub recipient2: Option<AccountInfo<'info>>,
    
    /// CHECK: 第三个接收方账户，如果有的话
    #[account(mut)]
    pub recipient3: Option<AccountInfo<'info>>,
    
    /// CHECK: 第四个接收方账户，如果有的话
    #[account(mut)]
    pub recipient4: Option<AccountInfo<'info>>,
}

pub fn handler(
    ctx: Context<Release>,
    payment_amounts: Vec<u64>,
) -> Result<()> {
    // 首先获取所有需要的数据，避免后续借用冲突
    let buyer = ctx.accounts.escrow_account.buyer;
    let seller = ctx.accounts.escrow_account.seller;
    let moderator_opt = ctx.accounts.escrow_account.moderator;
    let unique_id = ctx.accounts.escrow_account.unique_id;
    let bump = ctx.bumps.escrow_account;
    let token_type = ctx.accounts.escrow_account.token_type.clone();
    let amount = ctx.accounts.escrow_account.amount;
    let unlock_time = ctx.accounts.escrow_account.unlock_time;
    let required_signatures = ctx.accounts.escrow_account.required_signatures;
    let buyer_signed = ctx.accounts.escrow_account.buyer_signed;
    let seller_signed = ctx.accounts.escrow_account.seller_signed;
    let moderator_signed = ctx.accounts.escrow_account.moderator_signed;
    
    // 重要：提前获取escrow账户信息的克隆，避免后续借用冲突
    let escrow_account_info = ctx.accounts.escrow_account.to_account_info();
    
    // 验证签名和时间锁
    let current_time = ctx.accounts.clock.unix_timestamp;
    let signatures_count = buyer_signed as u8 + seller_signed as u8 + moderator_signed as u8;
    let time_expired = current_time >= unlock_time;
    
    if !time_expired && signatures_count < required_signatures {
        return err!(EscrowError::InsufficientSignatures);
    } else if time_expired && !seller_signed {
        // 确保即使时间锁过期，卖家仍需签名
        return err!(EscrowError::InsufficientSignatures);
    }
    
    // 验证支付目标
    require!(!payment_amounts.is_empty(), EscrowError::InvalidPaymentTargets);
    require!(payment_amounts.len() <= MAX_PAYMENT_TARGETS, 
            EscrowError::TooManyPaymentTargets);
    
    // 计算总金额和验证
    let mut total_payment: u64 = 0;
    for amount in &payment_amounts {
        total_payment = total_payment.checked_add(*amount)
            .ok_or(error!(EscrowError::AmountOverflow))?;
    }
    require!(total_payment == amount, EscrowError::PaymentAmountMismatch);

    // 创建签名种子
    let escrow_signer_seeds = &[
        ESCROW_SEED_PREFIX,
        buyer.as_ref(),
        seller.as_ref(),
        moderator_opt.as_ref().map_or(&[], |m| m.as_ref()),
        &unique_id,
        &[bump]
    ];
    let escrow_seeds = &[&escrow_signer_seeds[..]];
    
    // 现在可以安全修改escrow状态
    let escrow = &mut ctx.accounts.escrow_account;
    escrow.state = EscrowState::Completed;
    escrow.amount = 0;
    
    // 执行资金转移
    match token_type {
        TokenType::Sol => {
            // 处理SOL转账
            for (i, amount) in payment_amounts.iter().enumerate() {
                let recipient_info = match i {
                    0 => ctx.accounts.recipients.to_account_info(),
                    1 if ctx.accounts.recipient2.is_some() => ctx.accounts.recipient2.as_ref().unwrap().to_account_info(),
                    2 if ctx.accounts.recipient3.is_some() => ctx.accounts.recipient3.as_ref().unwrap().to_account_info(),
                    3 if ctx.accounts.recipient4.is_some() => ctx.accounts.recipient4.as_ref().unwrap().to_account_info(),
                    _ => return err!(EscrowError::InvalidPaymentTargets),
                };
                
                // 转出SOL
                let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
                    &escrow.key(),
                    &recipient_info.key(),
                    *amount,
                );
                
                anchor_lang::solana_program::program::invoke_signed(
                    &transfer_ix,
                    &[
                        escrow_account_info.clone(),
                        recipient_info.clone(),
                        ctx.accounts.system_program.to_account_info(),
                    ],
                    escrow_seeds,
                )?;
            }
        },
        TokenType::Spl(mint) => {
            // 处理SPL代币转账
            let token_program = ctx.accounts.token_program.as_ref()
                .ok_or(error!(EscrowError::InvalidTokenAccount))?;
            let escrow_token_account = ctx.accounts.escrow_token_account.as_ref()
                .ok_or(error!(EscrowError::InvalidTokenAccount))?;
            
            for (i, amount) in payment_amounts.iter().enumerate() {
                let recipient_info = match i {
                    0 => ctx.accounts.recipients.to_account_info(),
                    1 if ctx.accounts.recipient2.is_some() => ctx.accounts.recipient2.as_ref().unwrap().to_account_info(),
                    2 if ctx.accounts.recipient3.is_some() => ctx.accounts.recipient3.as_ref().unwrap().to_account_info(),
                    3 if ctx.accounts.recipient4.is_some() => ctx.accounts.recipient4.as_ref().unwrap().to_account_info(),
                    _ => return err!(EscrowError::InvalidPaymentTargets),
                };
                
                // 直接解析代币账户
                let recipient_token_account = SplTokenAccount::unpack(&recipient_info.try_borrow_data()?)?;
                require!(recipient_token_account.mint == mint, EscrowError::InvalidTokenAccount);
                
                // 转出SPL代币
                let transfer_accounts = Transfer {
                    from: escrow_token_account.to_account_info(),
                    to: recipient_info,
                    authority: ctx.accounts.escrow_account.to_account_info(),
                };
                
                let cpi_ctx = CpiContext::new_with_signer(
                    token_program.to_account_info(),
                    transfer_accounts,
                    escrow_seeds,
                );
                
                token::transfer(cpi_ctx, *amount)?;
            }
            
            // 关闭托管代币账户，将剩余租金退还给买家
            let close_accounts = CloseAccount {
                account: escrow_token_account.to_account_info(),
                destination: ctx.accounts.buyer.to_account_info(),
                authority: ctx.accounts.escrow_account.to_account_info(),
            };
            
            let cpi_ctx = CpiContext::new_with_signer(
                token_program.to_account_info(),
                close_accounts,
                escrow_seeds,
            );
            
            token::close_account(cpi_ctx)?;
        }
    }
    
    msg!("托管资金释放完成");
    Ok(())
} 