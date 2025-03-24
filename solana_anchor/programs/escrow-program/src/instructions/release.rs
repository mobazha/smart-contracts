use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, CloseAccount, Transfer};
use crate::{state::*, error::*};

#[derive(Accounts)]
#[instruction(payment_targets: Vec<PaymentTarget>)]
pub struct Release<'info> {
    #[account(
        constraint = (initiator.key() == escrow_account.buyer || initiator.key() == escrow_account.seller) @ EscrowError::InvalidSigner
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
    
    // SOL支付或SPL代币支付需要的额外账户
    pub token_program: Option<Program<'info, Token>>,
    
    #[account(
        mut,
        token::authority = escrow_account
    )]
    pub escrow_token_account: Option<Account<'info, TokenAccount>>,
    
    /// CHECK: 买家账户，用于接收SPL代币账户的租金
    #[account(mut)]
    pub buyer: Option<AccountInfo<'info>>,
    
    // 接收方账户需要在指令执行时动态检查，因为数量不固定
    /// CHECK: 所有接收方账户会在指令中进行验证
    pub remaining_accounts: Option<Vec<AccountInfo<'info>>>,
}

pub fn handler(
    ctx: Context<Release>,
    payment_targets: Vec<PaymentTarget>,
) -> Result<()> {
    // 验证付款目标数量
    require!(
        !payment_targets.is_empty(),
        EscrowError::EmptyPaymentTargets
    );
    require!(
        payment_targets.len() <= MAX_PAYMENT_TARGETS,
        EscrowError::TooManyPaymentTargets
    );
    
    let escrow = &mut ctx.accounts.escrow_account;
    
    // 检查签名数量
    let signed_count = escrow.buyer_signed as u8 
        + escrow.seller_signed as u8 
        + (escrow.moderator.is_some() && escrow.moderator_signed) as u8;
    
    // 检查签名数量或时间锁
    let time_lock_passed = escrow.unlock_time > 0 && 
        ctx.accounts.clock.unix_timestamp >= escrow.unlock_time;
    
    if signed_count < escrow.required_signatures {
        // 只有设置了时间锁时才检查
        if !time_lock_passed {
            return err!(EscrowError::InsufficientSignatures);
        }
        // 超过时间锁后，只需要卖家签名
        if !escrow.seller_signed {
            return err!(EscrowError::InsufficientSignatures);
        }
    }
    
    // 验证总金额
    let total_amount: u64 = payment_targets.iter()
        .map(|target| target.amount)
        .sum();
    require!(
        total_amount <= escrow.amount,
        EscrowError::AmountOverflow
    );
    
    // 根据代币类型处理资金释放
    match escrow.token_type {
        TokenType::Sol => {
            // 处理SOL释放
            for (i, target) in payment_targets.iter().enumerate() {
                if let Some(remaining_accounts) = &ctx.accounts.remaining_accounts {
                    if i < remaining_accounts.len() {
                        let recipient = &remaining_accounts[i];
                        **escrow_account.to_account_info().try_borrow_mut_lamports()? -= target.amount;
                        **recipient.try_borrow_mut_lamports()? += target.amount;
                    }
                }
            }
            
            // 关闭账户:将剩余的租金返还给买家
            if let Some(buyer) = &ctx.accounts.buyer {
                let remaining_lamports = escrow_account.to_account_info().lamports();
                **escrow_account.to_account_info().try_borrow_mut_lamports()? = 0;
                **buyer.try_borrow_mut_lamports()? += remaining_lamports;
            }
        },
        TokenType::Spl(_) => {
            // 处理SPL代币释放
            if let (Some(escrow_token_account), Some(token_program), Some(buyer), Some(remaining_accounts)) = (
                &ctx.accounts.escrow_token_account,
                &ctx.accounts.token_program,
                &ctx.accounts.buyer,
                &ctx.accounts.remaining_accounts,
            ) {
                // 转移代币给接收方
                for (i, target) in payment_targets.iter().enumerate() {
                    if i < remaining_accounts.len() {
                        let recipient_token_account = &remaining_accounts[i];
                        
                        let seeds = &[
                            ESCROW_SEED_PREFIX,
                            escrow.buyer.as_ref(),
                            escrow.seller.as_ref(),
                            escrow.moderator.as_ref().map_or(&[], |m| m.as_ref()),
                            &escrow.unique_id[..],
                            &[ctx.bumps.escrow_account],
                        ];
                        
                        let signer = &[&seeds[..]];
                        
                        // 转移代币
                        let cpi_accounts = Transfer {
                            from: escrow_token_account.to_account_info(),
                            to: recipient_token_account.to_account_info(),
                            authority: escrow_account.to_account_info(),
                        };
                        
                        let cpi_context = CpiContext::new_with_signer(
                            token_program.to_account_info(),
                            cpi_accounts,
                            signer,
                        );
                        
                        token::transfer(cpi_context, target.amount)?;
                    }
                }
                
                // 关闭代币账户，返还租金
                let seeds = &[
                    ESCROW_SEED_PREFIX,
                    escrow.buyer.as_ref(),
                    escrow.seller.as_ref(),
                    escrow.moderator.as_ref().map_or(&[], |m| m.as_ref()),
                    &escrow.unique_id[..],
                    &[ctx.bumps.escrow_account],
                ];
                
                let signer = &[&seeds[..]];
                
                let cpi_accounts = CloseAccount {
                    account: escrow_token_account.to_account_info(),
                    destination: buyer.to_account_info(),
                    authority: escrow_account.to_account_info(),
                };
                
                let cpi_context = CpiContext::new_with_signer(
                    token_program.to_account_info(),
                    cpi_accounts,
                    signer,
                );
                
                token::close_account(cpi_context)?;
            }
        }
    }
    
    // 更新托管状态
    escrow.state = EscrowState::Completed;
    
    msg!("资金释放由: {:?} 发起", ctx.accounts.initiator.key());
    
    Ok(())
} 