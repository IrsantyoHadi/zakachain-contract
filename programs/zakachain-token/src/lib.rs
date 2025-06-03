use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use zakachain_core::cpi::accounts::*;
use zakachain_core::program::ZakachainCore;

declare_id!("Hpfy7Cyo1mV4fHgU2vVPJ57FvGSRNNcpMPrrV2a2ugZf");

#[program]
pub mod zakachain_token {
    use super::*;

    pub fn receive_zakat(
        ctx: Context<ReceiveZakat>,
        amount: u64,
    ) -> Result<()> {
        let state = &ctx.accounts.state;
        require!(state.is_initialized, ZakaChainError::NotInitialized);

        // Calculate Amil fee
        let amil_fee = (amount as u128)
            .checked_mul(state.amil_fee_percentage as u128)
            .unwrap()
            .checked_div(1000)
            .unwrap() as u64;

        let net_amount = amount.checked_sub(amil_fee).unwrap();

        // Transfer tokens to the program's token account
        let transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.payer_token_account.to_account_info(),
                to: ctx.accounts.program_token_account.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        );
        token::transfer(transfer_ctx, net_amount)?;

        // Transfer Amil fee
        if amil_fee > 0 {
            let transfer_ctx = CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.payer_token_account.to_account_info(),
                    to: ctx.accounts.amil_token_account.to_account_info(),
                    authority: ctx.accounts.payer.to_account_info(),
                },
            );
            token::transfer(transfer_ctx, amil_fee)?;
        }

        // Update state through CPI
        let cpi_ctx = CpiContext::new(
            ctx.accounts.core_program.to_account_info(),
            UpdateCollection {
                state: ctx.accounts.state.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            },
        );
        zakachain_core::cpi::update_collection(cpi_ctx, net_amount, amil_fee)?;

        emit!(ZakatReceived {
            payer: ctx.accounts.payer.key(),
            amount: net_amount,
            amil_fee,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    pub fn withdraw_for_manual_distribution(
        ctx: Context<WithdrawForManualDistribution>,
        amount: u64,
        description: String,
    ) -> Result<()> {
        let state = &ctx.accounts.state;
        require!(state.is_initialized, ZakaChainError::NotInitialized);
        require!(
            ctx.accounts.amil.key() == state.amil,
            ZakaChainError::Unauthorized
        );
        require!(
            ctx.accounts.amil_operational_account.owner == ctx.accounts.amil.key(),
            ZakaChainError::Unauthorized
        );
        
        require!(
            ctx.accounts.program_token_account.amount >= amount,
            ZakaChainError::InsufficientFunds
        );

        require!(
            description.len() <= MAX_DESCRIPTION_LEN,
            ZakaChainError::DescriptionTooLong
        );

        require!(
            amount >= 100_000_000, // 100 USDC
            ZakaChainError::WithdrawalAmountTooSmall
        );

        require!(
            amount <= 100_000_000_000, // 100,000 USDC
            ZakaChainError::WithdrawalAmountTooLarge
        );

        let current_time = Clock::get()?.unix_timestamp;
        require!(
            current_time - state.last_withdrawal_timestamp >= 86400, // 24 hours
            ZakaChainError::WithdrawalCooldown
        );

        let seeds: &[&[u8]] = &[b"state"];
        let signer = &[seeds];
        
        let transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.program_token_account.to_account_info(),
                to: ctx.accounts.amil_operational_account.to_account_info(),
                authority: ctx.accounts.state.to_account_info(),
            },
            signer,
        );
        token::transfer(transfer_ctx, amount)?;

        // Update state through CPI
        let cpi_ctx = CpiContext::new(
            ctx.accounts.core_program.to_account_info(),
            UpdateWithdrawal {
                state: ctx.accounts.state.to_account_info(),
                authority: ctx.accounts.amil.to_account_info(),
            },
        );
        zakachain_core::cpi::update_withdrawal(cpi_ctx, current_time)?;

        emit!(ManualDistributionWithdrawal {
            amount,
            description,
            timestamp: current_time,
            withdrawal_count: state.manual_withdrawal_count + 1,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct ReceiveZakat<'info> {
    pub state: Account<'info, zakachain_core::ZakaChainState>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub payer_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub program_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub amil_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub core_program: Program<'info, ZakachainCore>,
}

#[derive(Accounts)]
pub struct WithdrawForManualDistribution<'info> {
    pub state: Account<'info, zakachain_core::ZakaChainState>,
    pub amil: Signer<'info>,
    #[account(
        mut,
        constraint = program_token_account.mint == amil_operational_account.mint @ ZakaChainError::InvalidTokenMint,
        constraint = program_token_account.owner == id() @ ZakaChainError::InvalidTokenAccount
    )]
    pub program_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = amil_operational_account.owner == amil.key() @ ZakaChainError::Unauthorized,
        constraint = amil_operational_account.mint == program_token_account.mint @ ZakaChainError::InvalidTokenMint
    )]
    pub amil_operational_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub core_program: Program<'info, ZakachainCore>,
}

#[event]
pub struct ZakatReceived {
    pub payer: Pubkey,
    pub amount: u64,
    pub amil_fee: u64,
    pub timestamp: i64,
}

#[event]
pub struct ManualDistributionWithdrawal {
    pub amount: u64,
    pub description: String,
    pub timestamp: i64,
    pub withdrawal_count: u32,
}

#[error_code]
pub enum ZakaChainError {
    #[msg("The contract has not been initialized")]
    NotInitialized,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Invalid token mint")]
    InvalidTokenMint,
    #[msg("Invalid token account")]
    InvalidTokenAccount,
    #[msg("Insufficient funds for withdrawal")]
    InsufficientFunds,
    #[msg("Description is too long (max 100 characters)")]
    DescriptionTooLong,
    #[msg("Withdrawal amount is too small (minimum 100 USDC)")]
    WithdrawalAmountTooSmall,
    #[msg("Withdrawal amount is too large (maximum 100,000 USDC)")]
    WithdrawalAmountTooLarge,
    #[msg("Must wait 24 hours between withdrawals")]
    WithdrawalCooldown,
}

pub const MAX_DESCRIPTION_LEN: usize = 100; 