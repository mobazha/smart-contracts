use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, CloseAccount, Transfer};
use crate::{state::*, error::*};

#[derive(Accounts)]
#[instruction(payment_targets: Vec<PaymentTarget>)]
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
        token::authority = escrow_account,
        constraint = {
            match escrow_account.token_type {
                TokenType::Spl(_) => true,
                _ => true
            }
        }
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
    pub recipient: AccountInfo<'info>,
}

pub fn handler(
    ctx: Context<Release>,
    payment_targets: Vec<PaymentTarget>,
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
    }
    
    // 验证支付目标
    require!(!payment_targets.is_empty(), EscrowError::InvalidPaymentTargets);
    require!(payment_targets.len() <= MAX_PAYMENT_TARGETS, 
            EscrowError::TooManyPaymentTargets);
    
    // 计算总金额和验证
    let mut total_payment: u64 = 0;
    for target in &payment_targets {
        total_payment = total_payment.checked_add(target.amount)
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
            for target in &payment_targets {
                let recipient_info = &ctx.accounts.recipient;
                require!(recipient_info.key() == target.recipient, 
                         EscrowError::InvalidPaymentTargets);
                
                // 转出SOL
                let transfer_ix = anchor_lang::solana_program::system_instruction::transfer(
                    &escrow.key(),
                    &target.recipient,
                    target.amount,
                );
                
                anchor_lang::solana_program::program::invoke_signed(
                    &transfer_ix,
                    &[
                        escrow_account_info.clone(),  // 使用克隆的账户信息
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
            
            for target in &payment_targets {
                // 在实际应用中，您需要对每个接收者的代币账户进行验证
                // 这里简化处理，假设接收者账户已正确传入
                let recipient_info = &ctx.accounts.recipient;
                require!(recipient_info.key() == target.recipient, 
                         EscrowError::InvalidPaymentTargets);
                
                // 转出SPL代币
                let transfer_accounts = Transfer {
                    from: escrow_token_account.to_account_info(),
                    to: recipient_info.clone(),
                    authority: ctx.accounts.escrow_account.to_account_info(),
                };
                
                let cpi_ctx = CpiContext::new_with_signer(
                    token_program.to_account_info(),
                    transfer_accounts,
                    escrow_seeds,
                );
                
                token::transfer(cpi_ctx, target.amount)?;
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