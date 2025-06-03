use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use anchor_spl::associated_token::AssociatedToken;
use std::str::FromStr;
use borsh::{BorshSerialize, BorshDeserialize};

declare_id!("3EJSTPJYM3BaNBvL7haWnhXoNh5GvmsQfwL1QQ2am3GJ");

pub const MAX_DESCRIPTION_LEN: usize = 100;
pub const PROGRAM_DEPLOYER: &str = "F2sKSFqHi4NXsez7dfafW3rCA9YTCK5aNMrCZsNdoq9j";

#[program]
pub mod zakachain {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        amil_fee_percentage: u8,
    ) -> Result<()> {
        require!(
            ctx.accounts.amil.key() == Pubkey::from_str(PROGRAM_DEPLOYER).unwrap(),
            ZakaChainError::Unauthorized
        );
        require!(amil_fee_percentage <= 125, ZakaChainError::InvalidFeePercentage);
        let state = &mut ctx.accounts.state;
        state.amil = ctx.accounts.amil.key();
        state.amil_fee_percentage = amil_fee_percentage;
        state.total_zakat_collected = 0;
        state.total_zakat_distributed = 0;
        state.total_amil_fees_collected = 0;
        state.is_initialized = true;
        state.last_withdrawal_timestamp = 0;
        state.manual_withdrawal_count = 0;
        state.withdrawal_count = 0;
        Ok(())
    }

    pub fn update_authority(ctx: Context<UpdateAuthority>, new_authority: Pubkey) -> Result<()> {
        require!(ctx.accounts.state.is_initialized, ZakaChainError::NotInitialized);
        require!(ctx.accounts.authority.key() == ctx.accounts.state.amil, ZakaChainError::Unauthorized);
        ctx.accounts.state.amil = new_authority;
        Ok(())
    }

    pub fn update_fee_percentage(ctx: Context<UpdateFeePercentage>, new_fee_percentage: u8) -> Result<()> {
        require!(ctx.accounts.state.is_initialized, ZakaChainError::NotInitialized);
        require!(ctx.accounts.authority.key() == ctx.accounts.state.amil, ZakaChainError::Unauthorized);
        require!(new_fee_percentage <= 125, ZakaChainError::InvalidFeePercentage);
        ctx.accounts.state.amil_fee_percentage = new_fee_percentage;
        Ok(())
    }

    pub fn add_mustahik(
        ctx: Context<AddMustahik>,
        unique_id: String,
        name: String,
    ) -> Result<()> {
        let state = &mut ctx.accounts.state;
        require!(state.is_initialized, ZakaChainError::NotInitialized);
        require!(ctx.accounts.amil.key() == state.amil, ZakaChainError::Unauthorized);
        
        let mustahik_account = &mut ctx.accounts.mustahik_account;
        mustahik_account.address = ctx.accounts.mustahik.key();
        mustahik_account.unique_id = unique_id;
        mustahik_account.name = name;
        mustahik_account.is_active = true;
        mustahik_account.created_at = Clock::get()?.unix_timestamp;
        mustahik_account.updated_at = Clock::get()?.unix_timestamp;
        
        emit!(MustahikAdded {
            mustahik: ctx.accounts.mustahik.key(),
            mustahik_account: mustahik_account.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    pub fn remove_mustahik(ctx: Context<RemoveMustahik>) -> Result<()> {
        let state = &mut ctx.accounts.state;
        require!(state.is_initialized, ZakaChainError::NotInitialized);
        require!(ctx.accounts.amil.key() == state.amil, ZakaChainError::Unauthorized);
        
        let mustahik_account = &mut ctx.accounts.mustahik_account;
        require!(mustahik_account.is_active, ZakaChainError::InvalidMustahik);
        mustahik_account.is_active = false;
        mustahik_account.updated_at = Clock::get()?.unix_timestamp;
        
        emit!(MustahikRemoved {
            mustahik: mustahik_account.address,
            mustahik_account: mustahik_account.key(),
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    pub fn collect_zakat(
        ctx: Context<CollectZakat>,
        amount: u64,
    ) -> Result<()> {
        let state = &mut ctx.accounts.state;
        require!(state.is_initialized, ZakaChainError::NotInitialized);
        let amil_fee = (amount as u128)
            .checked_mul(state.amil_fee_percentage as u128)
            .unwrap()
            .checked_div(1000)
            .unwrap() as u64;
        let net_amount = amount.checked_sub(amil_fee).unwrap();
        let transfer_accounts = Transfer {
            from: ctx.accounts.payer_token_account.to_account_info(),
            to: ctx.accounts.program_token_account.to_account_info(),
            authority: ctx.accounts.payer.to_account_info(),
        };
        token::transfer(
            CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_accounts),
            net_amount,
        )?;
        if amil_fee > 0 {
            let transfer_accounts = Transfer {
                from: ctx.accounts.payer_token_account.to_account_info(),
                to: ctx.accounts.amil_token_account.to_account_info(),
                authority: ctx.accounts.payer.to_account_info(),
            };
            token::transfer(
                CpiContext::new(ctx.accounts.token_program.to_account_info(), transfer_accounts),
                amil_fee,
            )?;
        }
        state.total_zakat_collected = state.total_zakat_collected.checked_add(net_amount).unwrap();
        state.total_amil_fees_collected = state.total_amil_fees_collected.checked_add(amil_fee).unwrap();
        emit!(ZakatCollected {
            payer: ctx.accounts.payer.key(),
            amount: net_amount,
            amil_fee,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    pub fn distribute_to_mustahik(
        ctx: Context<DistributeToMustahik>,
        amount: u64,
    ) -> Result<()> {
        let state_info = ctx.accounts.state.to_account_info();
        let state = &mut ctx.accounts.state;
        require!(state.is_initialized, ZakaChainError::NotInitialized);
        require!(ctx.accounts.amil.key() == state.amil, ZakaChainError::Unauthorized);
        require!(ctx.accounts.mustahik_account.is_active, ZakaChainError::InvalidMustahik);
        
        let seeds: &[&[u8]] = &[b"state"];
        let signer = &[seeds];
        let transfer_accounts = Transfer {
            from: ctx.accounts.program_token_account.to_account_info(),
            to: ctx.accounts.mustahik_token_account.to_account_info(),
            authority: state_info,
        };
        token::transfer(
            CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), transfer_accounts, signer),
            amount,
        )?;
        state.total_zakat_distributed = state.total_zakat_distributed.checked_add(amount).unwrap();
        emit!(ZakatDistributed {
            mustahik: ctx.accounts.mustahik.key(),
            amount,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    pub fn withdraw_amil_fees(
        ctx: Context<WithdrawAmilFees>,
        amount: u64,
    ) -> Result<()> {
        let state_info = ctx.accounts.state.to_account_info();
        let state = &mut ctx.accounts.state;
        require!(state.is_initialized, ZakaChainError::NotInitialized);
        require!(ctx.accounts.amil.key() == state.amil, ZakaChainError::Unauthorized);
        require!(amount <= state.total_amil_fees_collected, ZakaChainError::InsufficientFunds);
        let seeds: &[&[u8]] = &[b"state"];
        let signer = &[seeds];
        let transfer_accounts = Transfer {
            from: ctx.accounts.program_token_account.to_account_info(),
            to: ctx.accounts.amil_token_account.to_account_info(),
            authority: state_info,
        };
        token::transfer(
            CpiContext::new_with_signer(ctx.accounts.token_program.to_account_info(), transfer_accounts, signer),
            amount,
        )?;
        state.total_amil_fees_collected = state.total_amil_fees_collected.checked_sub(amount).unwrap();
        state.last_withdrawal_timestamp = Clock::get()?.unix_timestamp;
        state.manual_withdrawal_count = state.manual_withdrawal_count.checked_add(1).unwrap();
        state.withdrawal_count = state.withdrawal_count.checked_add(1).unwrap();
        emit!(AmilFeesWithdrawn {
            amil: ctx.accounts.amil.key(),
            amount,
            timestamp: Clock::get()?.unix_timestamp,
        });
        Ok(())
    }

    pub fn withdraw_zakat_manual(
        ctx: Context<WithdrawZakatManual>,
        amount: u64,
        unique_id: String,
    ) -> Result<()> {
        // Get state info before mutable borrow
        let state_info = ctx.accounts.state.to_account_info();
        
        let state = &mut ctx.accounts.state;
        require!(state.is_initialized, ZakaChainError::NotInitialized);
        require!(state.amil == ctx.accounts.amil.key(), ZakaChainError::Unauthorized);
        require!(amount > 0, ZakaChainError::InvalidAmount);
        require!(unique_id.len() <= MAX_DESCRIPTION_LEN, ZakaChainError::UniqueIdTooLong);

        // Verify recipient is amil
        require!(
            ctx.accounts.recipient_token_account.owner == ctx.accounts.amil.key(),
            ZakaChainError::InvalidRecipient
        );

        // Calculate withdrawal amount
        let withdrawal_amount = amount;

        // Transfer tokens
        let transfer_ctx = CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.program_token_account.to_account_info(),
                to: ctx.accounts.recipient_token_account.to_account_info(),
                authority: state_info,
            },
        );

        token::transfer(transfer_ctx, withdrawal_amount)?;

        // Update state
        state.total_zakat_distributed = state.total_zakat_distributed.checked_add(withdrawal_amount).unwrap();
        state.withdrawal_count = state.withdrawal_count.checked_add(1).unwrap();
        state.manual_withdrawal_count = state.manual_withdrawal_count.checked_add(1).unwrap();
        state.last_withdrawal_timestamp = Clock::get()?.unix_timestamp;

        // Emit event
        emit!(ZakatWithdrawn {
            amount: withdrawal_amount,
            unique_id,
            timestamp: Clock::get()?.unix_timestamp,
            withdrawal_count: state.withdrawal_count,
        });

        Ok(())
    }

    pub fn view_mustahik_status(ctx: Context<GetMustahikStatus>) -> Result<MustahikStatusView> {
        let mustahik_account = &ctx.accounts.mustahik_account;
        Ok(MustahikStatusView {
            address: mustahik_account.address,
            unique_id: mustahik_account.unique_id.clone(),
            name: mustahik_account.name.clone(),
            is_active: mustahik_account.is_active,
            created_at: mustahik_account.created_at,
            updated_at: mustahik_account.updated_at,
        })
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

#[derive(Accounts)]
pub struct AddMustahik<'info> {
    #[account(mut)]
    pub state: Account<'info, ZakaChainState>,
    #[account(mut)]
    pub amil: Signer<'info>,
    /// CHECK: This is the mustahik's wallet address
    pub mustahik: AccountInfo<'info>,
    #[account(
        init,
        payer = amil,
        space = 8 + MustahikAccount::LEN,
        seeds = [b"mustahik", mustahik.key().as_ref()],
        bump
    )]
    pub mustahik_account: Account<'info, MustahikAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RemoveMustahik<'info> {
    #[account(mut)]
    pub state: Account<'info, ZakaChainState>,
    pub amil: Signer<'info>,
    #[account(
        mut,
        seeds = [b"mustahik", mustahik_account.address.as_ref()],
        bump
    )]
    pub mustahik_account: Account<'info, MustahikAccount>,
}

#[derive(Accounts)]
pub struct CollectZakat<'info> {
    #[account(mut)]
    pub state: Account<'info, ZakaChainState>,
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(mut)]
    pub payer_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub program_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub amil_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct DistributeToMustahik<'info> {
    #[account(mut)]
    pub state: Account<'info, ZakaChainState>,
    pub amil: Signer<'info>,
    /// CHECK: This is the mustahik's wallet address
    pub mustahik: AccountInfo<'info>,
    #[account(
        seeds = [b"mustahik", mustahik.key().as_ref()],
        bump
    )]
    pub mustahik_account: Account<'info, MustahikAccount>,
    #[account(mut)]
    pub program_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub mustahik_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct WithdrawAmilFees<'info> {
    #[account(mut)]
    pub state: Account<'info, ZakaChainState>,
    pub amil: Signer<'info>,
    #[account(mut)]
    pub program_token_account: Account<'info, TokenAccount>,
    #[account(mut)]
    pub amil_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(Accounts)]
pub struct GetMustahikStatus<'info> {
    /// CHECK: This is the mustahik's wallet address
    pub mustahik: AccountInfo<'info>,
    #[account(
        seeds = [b"mustahik", mustahik.key().as_ref()],
        bump
    )]
    pub mustahik_account: Account<'info, MustahikAccount>,
}

#[derive(Accounts)]
pub struct WithdrawZakatManual<'info> {
    #[account(mut)]
    pub state: Account<'info, ZakaChainState>,
    #[account(mut)]
    pub amil: Signer<'info>,
    #[account(mut)]
    pub program_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = recipient_token_account.owner == amil.key() @ ZakaChainError::InvalidRecipient
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

#[derive(BorshSerialize, BorshDeserialize, Clone)]
pub struct Withdrawal {
    pub id: u32,
    pub amount: u64,
    pub timestamp: i64,
    pub total_zakat_at_withdrawal: u64,
    pub is_reported: bool,
    pub description: String,
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
    pub withdrawal_count: u32,
}

impl ZakaChainState {
    pub const LEN: usize = 32 + 1 + 8 + 8 + 8 + 8 + 4 + 4;
}

#[account]
pub struct MustahikAccount {
    pub address: Pubkey,
    pub unique_id: String,
    pub name: String,
    pub is_active: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

impl MustahikAccount {
    pub const LEN: usize = 32 + 100 + 100 + 1 + 8 + 8; // pubkey + unique_id + name + is_active + timestamps
}

#[event]
pub struct ZakatCollected {
    pub payer: Pubkey,
    pub amount: u64,
    pub amil_fee: u64,
    pub timestamp: i64,
}

#[event]
pub struct MustahikAdded {
    pub mustahik: Pubkey,
    pub mustahik_account: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct MustahikRemoved {
    pub mustahik: Pubkey,
    pub mustahik_account: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct ZakatDistributed {
    pub mustahik: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct AmilFeesWithdrawn {
    pub amil: Pubkey,
    pub amount: u64,
    pub timestamp: i64,
}

#[event]
pub struct ZakatWithdrawn {
    pub amount: u64,
    pub unique_id: String,
    pub timestamp: i64,
    pub withdrawal_count: u32,
}

#[error_code]
pub enum ZakaChainError {
    #[msg("The contract has not been initialized")]
    NotInitialized,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Invalid fee percentage")]
    InvalidFeePercentage,
    #[msg("Invalid mustahik address")]
    InvalidMustahik,
    #[msg("Insufficient funds")]
    InsufficientFunds,
    #[msg("Description is too long")]
    DescriptionTooLong,
    #[msg("Invalid recipient - must be the amil's token account")]
    InvalidRecipient,
    #[msg("Invalid amount")]
    InvalidAmount,
    #[msg("Unique ID is too long")]
    UniqueIdTooLong,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MustahikStatusView {
    pub address: Pubkey,
    pub unique_id: String,
    pub name: String,
    pub is_active: bool,
    pub created_at: i64,
    pub updated_at: i64,
} 