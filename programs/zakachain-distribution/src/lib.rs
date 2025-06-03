use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use anchor_spl::associated_token::AssociatedToken;
use zakachain_core::program::ZakachainCore;
use zakachain_core::ZakaChainState;
use zakachain_core::cpi::accounts::UpdateDistribution;
use std::str::FromStr;

declare_id!("Hpfy7Cyo1mV4fHgU2vVPJ57FvGSRNNcpMPrrV2a2ugZf");

pub const MAX_MUSTAHIKS: usize = 50;
pub const MAX_DESCRIPTION_LEN: usize = 100;
pub const PROGRAM_DEPLOYER: &str = "F2sKSFqHi4NXsez7dfafW3rCA9YTCK5aNMrCZsNdoq9j";

#[program]
pub mod zakachain_distribution {
    use super::*;

    pub fn add_mustahik(
        ctx: Context<AddMustahik>,
        name: String,
        address: Pubkey,
    ) -> Result<()> {
        let state = &ctx.accounts.state;
        require!(state.is_initialized, ZakaChainError::NotInitialized);
        require!(
            ctx.accounts.authority.key() == state.amil || 
            ctx.accounts.authority.key() == Pubkey::from_str(PROGRAM_DEPLOYER).unwrap(),
            ZakaChainError::Unauthorized
        );
        require!(
            ctx.accounts.mustahik_list.mustahiks.len() < MAX_MUSTAHIKS,
            ZakaChainError::MaxMustahiksReached
        );
        let mustahik = Mustahik {
            name,
            address,
            total_received: 0,
        };
        ctx.accounts.mustahik_list.mustahiks.push(mustahik);
        Ok(())
    }

    pub fn remove_mustahik(ctx: Context<RemoveMustahik>, index: usize) -> Result<()> {
        let state = &ctx.accounts.state;
        require!(state.is_initialized, ZakaChainError::NotInitialized);
        require!(
            ctx.accounts.authority.key() == state.amil || 
            ctx.accounts.authority.key() == Pubkey::from_str(PROGRAM_DEPLOYER).unwrap(),
            ZakaChainError::Unauthorized
        );
        require!(
            index < ctx.accounts.mustahik_list.mustahiks.len(),
            ZakaChainError::InvalidIndex
        );
        ctx.accounts.mustahik_list.mustahiks.remove(index);
        Ok(())
    }

    pub fn distribute_to_mustahik(
        ctx: Context<DistributeToMustahik>,
        amount: u64,
    ) -> Result<()> {
        let state = &ctx.accounts.state;
        require!(state.is_initialized, ZakaChainError::NotInitialized);
        require!(
            ctx.accounts.authority.key() == state.amil || 
            ctx.accounts.authority.key() == Pubkey::from_str(PROGRAM_DEPLOYER).unwrap(),
            ZakaChainError::Unauthorized
        );
        let mustahik_list = &ctx.accounts.mustahik_list;
        require!(
            mustahik_list.mustahiks.iter().any(|m| m.address == ctx.accounts.mustahik.key()),
            ZakaChainError::InvalidMustahik
        );
        // Transfer tokens to mustahik
        let seeds: &[&[u8]] = &[b"state"];
        let signer = &[seeds];
        let transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.program_token_account.to_account_info(),
                to: ctx.accounts.mustahik_token_account.to_account_info(),
                authority: ctx.accounts.state.to_account_info(),
            },
            signer,
        );
        token::transfer(transfer_ctx, amount)?;
        // Update state through CPI
        let cpi_ctx = CpiContext::new(
            ctx.accounts.core_program.to_account_info(),
            UpdateDistribution {
                state: ctx.accounts.state.to_account_info(),
                authority: ctx.accounts.authority.to_account_info(),
            },
        );
        zakachain_core::cpi::update_distribution(cpi_ctx, amount)?;
        emit!(ZakatDistributed {
            mustahik: ctx.accounts.mustahik.key(),
            amount,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }
}

#[derive(Accounts)]
pub struct AddMustahik<'info> {
    pub state: Account<'info, ZakaChainState>,
    #[account(mut)]
    pub mustahik_list: Account<'info, MustahikList>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct RemoveMustahik<'info> {
    pub state: Account<'info, ZakaChainState>,
    #[account(mut)]
    pub mustahik_list: Account<'info, MustahikList>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct DistributeToMustahik<'info> {
    #[account(mut)]
    pub state: Account<'info, ZakaChainState>,
    #[account(mut)]
    pub mustahik_list: Account<'info, MustahikList>,
    pub authority: Signer<'info>,
    /// CHECK: This is the mustahik's wallet address
    pub mustahik: AccountInfo<'info>,
    #[account(mut)]
    pub program_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub mustahik_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub core_program: Program<'info, ZakachainCore>,
}

#[account]
pub struct MustahikList {
    pub mustahiks: Vec<Mustahik>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct Mustahik {
    pub name: String,
    pub address: Pubkey,
    pub total_received: u64,
}

#[event]
pub struct ZakatDistributed {
    pub mustahik: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[error_code]
pub enum ZakaChainError {
    #[msg("The contract has not been initialized")]
    NotInitialized,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Maximum number of mustahiks reached")]
    MaxMustahiksReached,
    #[msg("Invalid mustahik index")]
    InvalidIndex,
    #[msg("Invalid mustahik address")]
    InvalidMustahik,
} 