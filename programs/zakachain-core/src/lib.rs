use anchor_lang::prelude::*;
use std::str::FromStr;

declare_id!("375fwyjaiv8qfFhhBd8kbHnQy5VVKxEBgKMCGDkbxUZm");

pub const PROGRAM_DEPLOYER: &str = "F2sKSFqHi4NXsez7dfafW3rCA9YTCK5aNMrCZsNdoq9j";

// CPI Account Structs
#[derive(Accounts)]
pub struct UpdateCollection<'info> {
    #[account(mut)]
    pub state: Account<'info, ZakaChainState>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateWithdrawal<'info> {
    #[account(mut)]
    pub state: Account<'info, ZakaChainState>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateDistribution<'info> {
    #[account(mut)]
    pub state: Account<'info, ZakaChainState>,
    pub authority: Signer<'info>,
}

// State Management Module
pub mod state {
    use anchor_lang::prelude::*;

    #[account]
    pub struct State {
        pub is_initialized: bool,
        pub authority: Pubkey,
        pub program_deployer: Pubkey,
        pub fee_percentage: u8,
        pub total_distributed: u64,
        pub total_fees_collected: u64,
    }

    impl State {
        pub const LEN: usize = 1 + 32 + 32 + 1 + 8 + 8;
    }
}

// Error Handling
#[error_code]
pub enum ZakaChainError {
    #[msg("State account is not initialized")]
    NotInitialized,
    #[msg("State account is already initialized")]
    AlreadyInitialized,
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("Invalid fee percentage")]
    InvalidFeePercentage,
}

#[program]
pub mod zakachain_core {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        amil_fee_percentage: u8,
    ) -> Result<()> {
        require!(
            ctx.accounts.amil.key() == Pubkey::from_str(PROGRAM_DEPLOYER).unwrap(),
            ZakaChainError::Unauthorized
        );

        require!(
            amil_fee_percentage <= 125, // 12.5% maximum
            ZakaChainError::InvalidFeePercentage
        );

        let state = &mut ctx.accounts.state;
        state.amil = ctx.accounts.amil.key();
        state.amil_fee_percentage = amil_fee_percentage;
        state.total_zakat_collected = 0;
        state.total_zakat_distributed = 0;
        state.total_amil_fees_collected = 0;
        state.is_initialized = true;
        state.last_withdrawal_timestamp = 0;
        state.manual_withdrawal_count = 0;

        Ok(())
    }

    pub fn update_authority(ctx: Context<UpdateAuthority>, new_authority: Pubkey) -> Result<()> {
        require!(ctx.accounts.state.is_initialized, ZakaChainError::NotInitialized);
        require!(
            ctx.accounts.authority.key() == ctx.accounts.state.amil,
            ZakaChainError::Unauthorized
        );

        ctx.accounts.state.amil = new_authority;
        Ok(())
    }

    pub fn update_fee_percentage(ctx: Context<UpdateFeePercentage>, new_fee_percentage: u8) -> Result<()> {
        require!(ctx.accounts.state.is_initialized, ZakaChainError::NotInitialized);
        require!(
            ctx.accounts.authority.key() == ctx.accounts.state.amil,
            ZakaChainError::Unauthorized
        );
        require!(new_fee_percentage <= 125, ZakaChainError::InvalidFeePercentage);

        ctx.accounts.state.amil_fee_percentage = new_fee_percentage;
        Ok(())
    }

    pub fn update_collection(ctx: Context<UpdateCollection>, amount: u64, amil_fee: u64) -> Result<()> {
        let state = &mut ctx.accounts.state;
        require!(state.is_initialized, ZakaChainError::NotInitialized);
        
        state.total_zakat_collected = state.total_zakat_collected.checked_add(amount).unwrap();
        state.total_amil_fees_collected = state.total_amil_fees_collected.checked_add(amil_fee).unwrap();
        
        Ok(())
    }

    pub fn update_withdrawal(ctx: Context<UpdateWithdrawal>, timestamp: i64) -> Result<()> {
        let state = &mut ctx.accounts.state;
        require!(state.is_initialized, ZakaChainError::NotInitialized);
        
        state.last_withdrawal_timestamp = timestamp;
        state.manual_withdrawal_count = state.manual_withdrawal_count.checked_add(1).unwrap();
        
        Ok(())
    }

    pub fn update_distribution(ctx: Context<UpdateDistribution>, amount: u64) -> Result<()> {
        let state = &mut ctx.accounts.state;
        require!(state.is_initialized, ZakaChainError::NotInitialized);
        require!(ctx.accounts.authority.key() == state.amil, ZakaChainError::Unauthorized);
        
        state.total_zakat_distributed = state.total_zakat_distributed.checked_add(amount).unwrap();
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = amil,
        space = 8 + ZakaChainState::LEN,
        seeds = [b"state"],
        bump
    )]
    pub state: Account<'info, ZakaChainState>,
    #[account(mut)]
    pub amil: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateAuthority<'info> {
    #[account(mut)]
    pub state: Account<'info, ZakaChainState>,
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateFeePercentage<'info> {
    #[account(mut)]
    pub state: Account<'info, ZakaChainState>,
    pub authority: Signer<'info>,
}

#[account]
pub struct ZakaChainState {
    pub amil: Pubkey,
    pub amil_fee_percentage: u8,
    pub total_zakat_collected: u64,
    pub total_zakat_distributed: u64,
    pub total_amil_fees_collected: u64,
    pub is_initialized: bool,
    pub last_withdrawal_timestamp: i64,
    pub manual_withdrawal_count: u32,
}

impl ZakaChainState {
    pub const LEN: usize = 32 + // amil
        1 + // amil_fee_percentage
        8 + // total_zakat_collected
        8 + // total_zakat_distributed
        8 + // total_amil_fees_collected
        1 + // is_initialized
        8 + // last_withdrawal_timestamp
        4; // manual_withdrawal_count
} 