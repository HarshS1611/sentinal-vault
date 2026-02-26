use anchor_lang::{
    prelude::*,
    system_program::{transfer, Transfer},
};

declare_id!("2QLVKGpugTttecUSjjt4kERsVVrmhyzqMR6N5Cdp6q1H");

#[program]
pub mod sentinal_vault {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        cooldown_seconds: i64,
        inactivity_window_seconds: i64,
    ) -> Result<()> {
        let clock = Clock::get()?;

        // fund vault PDA to rent-exempt (SystemAccount cannot be init)
        let rent = Rent::get()?.minimum_balance(0);

        let cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        );
        transfer(cpi_ctx, rent)?;

        let state = &mut ctx.accounts.vault_state;
        state.owner = ctx.accounts.user.key();
        state.cooldown_seconds = cooldown_seconds;
        state.inactivity_window_seconds = inactivity_window_seconds;
        state.last_check_in = clock.unix_timestamp;
        state.last_withdraw = 0;
        state.total_deposited = 0;
        state.total_withdrawn = 0;
        state.vault_bump = ctx.bumps.vault;
        state.state_bump = ctx.bumps.vault_state;

        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        let cpi_ctx = CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user.to_account_info(),
                to: ctx.accounts.vault.to_account_info(),
            },
        );
        transfer(cpi_ctx, amount)?;
        ctx.accounts.vault_state.total_deposited += amount;
        Ok(())
    }

    pub fn check_in(ctx: Context<CheckIn>) -> Result<()> {
        ctx.accounts.vault_state.last_check_in = Clock::get()?.unix_timestamp;
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let clock = Clock::get()?;
        let state = &mut ctx.accounts.vault_state;

        require_keys_eq!(
            ctx.accounts.user.key(),
            state.owner,
            VaultError::Unauthorized
        );

        require!(
            clock.unix_timestamp - state.last_check_in
                <= state.inactivity_window_seconds,
            VaultError::InactiveUser
        );

        require!(
            clock.unix_timestamp - state.last_withdraw >= state.cooldown_seconds,
            VaultError::CooldownActive
        );

        require!(
            state.total_withdrawn + amount <= state.total_deposited,
            VaultError::InsufficientVaultBalance
        );

        let vault_state_key = state.key();
        let seeds = &[
            b"vault",
            vault_state_key.as_ref(),
            &[state.vault_bump],
        ];
        let signer_seeds: [&[&[u8]]; 1] = [&seeds[..]];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.system_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.user.to_account_info(),
            },
            &signer_seeds,
        );

        transfer(cpi_ctx, amount)?;

        state.last_withdraw = clock.unix_timestamp;
        state.total_withdrawn += amount;
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        seeds = [b"state", user.key().as_ref()],
        bump,
        space = 8 + VaultState::INIT_SPACE
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CheckIn<'info> {
    pub owner: Signer<'info>,

    #[account(
        mut,
        has_one = owner,
        seeds = [b"state", owner.key().as_ref()],
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [b"state", user.key().as_ref()],
        bump = vault_state.state_bump
    )]
    pub vault_state: Account<'info, VaultState>,

    #[account(
        mut,
        seeds = [b"vault", vault_state.key().as_ref()],
        bump = vault_state.vault_bump
    )]
    pub vault: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
#[derive(InitSpace)]
pub struct VaultState {
    pub owner: Pubkey,
    pub cooldown_seconds: i64,
    pub inactivity_window_seconds: i64,
    pub last_check_in: i64,
    pub last_withdraw: i64,
    pub total_deposited: u64,
    pub total_withdrawn: u64,
    pub vault_bump: u8,
    pub state_bump: u8,
}

#[error_code]
pub enum VaultError {
    #[msg("User has been inactive too long")]
    InactiveUser,
    #[msg("Cooldown period has not passed")]
    CooldownActive,
    #[msg("Insufficient vault balance")]
    InsufficientVaultBalance,
    #[msg("Unauthorized")]
    Unauthorized,
}