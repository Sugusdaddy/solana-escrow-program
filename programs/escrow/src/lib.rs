use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

declare_id!("EscrowProgram111111111111111111111111111111");

#[program]
pub mod escrow {
    use super::*;

    /// Initialize a new escrow
    pub fn initialize(
        ctx: Context<Initialize>,
        amount_a: u64,
        amount_b: u64,
        expiration: i64,
    ) -> Result<()> {
        require!(amount_a > 0, EscrowError::InvalidAmount);
        require!(amount_b > 0, EscrowError::InvalidAmount);
        
        let clock = Clock::get()?;
        require!(expiration > clock.unix_timestamp, EscrowError::InvalidExpiration);

        let escrow = &mut ctx.accounts.escrow;
        escrow.maker = ctx.accounts.maker.key();
        escrow.mint_a = ctx.accounts.mint_a.key();
        escrow.mint_b = ctx.accounts.mint_b.key();
        escrow.amount_a = amount_a;
        escrow.amount_b = amount_b;
        escrow.vault = ctx.accounts.vault.key();
        escrow.expiration = expiration;
        escrow.state = EscrowState::Initialized;
        escrow.bump = ctx.bumps.escrow;

        Ok(())
    }

    /// Maker deposits tokens into escrow vault
    pub fn deposit(ctx: Context<Deposit>) -> Result<()> {
        let escrow = &mut ctx.accounts.escrow;
        require!(escrow.state == EscrowState::Initialized, EscrowError::EscrowNotActive);

        // Transfer tokens from maker to vault
        let cpi_accounts = Transfer {
            from: ctx.accounts.maker_token_a.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.maker.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        token::transfer(cpi_ctx, escrow.amount_a)?;

        escrow.state = EscrowState::Active;

        emit!(EscrowDeposited {
            escrow: ctx.accounts.escrow.key(),
            maker: escrow.maker,
            amount: escrow.amount_a,
        });

        Ok(())
    }

    /// Taker exchanges tokens, completing the escrow
    pub fn exchange(ctx: Context<Exchange>) -> Result<()> {
        let escrow = &ctx.accounts.escrow;
        
        require!(escrow.state == EscrowState::Active, EscrowError::EscrowNotActive);
        
        let clock = Clock::get()?;
        require!(clock.unix_timestamp < escrow.expiration, EscrowError::EscrowExpired);

        // Transfer token B from taker to maker
        let cpi_accounts_b = Transfer {
            from: ctx.accounts.taker_token_b.to_account_info(),
            to: ctx.accounts.maker_token_b.to_account_info(),
            authority: ctx.accounts.taker.to_account_info(),
        };
        let cpi_ctx_b = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts_b);
        token::transfer(cpi_ctx_b, escrow.amount_b)?;

        // Transfer token A from vault to taker (using PDA authority)
        let escrow_key = ctx.accounts.escrow.key();
        let seeds = &[
            b"escrow",
            escrow_key.as_ref(),
            &[escrow.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts_a = Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.taker_token_a.to_account_info(),
            authority: ctx.accounts.escrow.to_account_info(),
        };
        let cpi_ctx_a = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts_a,
            signer_seeds,
        );
        token::transfer(cpi_ctx_a, escrow.amount_a)?;

        // Update state
        let escrow = &mut ctx.accounts.escrow;
        escrow.state = EscrowState::Completed;

        emit!(EscrowCompleted {
            escrow: ctx.accounts.escrow.key(),
            maker: escrow.maker,
            taker: ctx.accounts.taker.key(),
            amount_a: escrow.amount_a,
            amount_b: escrow.amount_b,
        });

        Ok(())
    }

    /// Maker cancels escrow and withdraws tokens
    pub fn cancel(ctx: Context<Cancel>) -> Result<()> {
        let escrow = &ctx.accounts.escrow;
        
        require!(
            escrow.state == EscrowState::Active || escrow.state == EscrowState::Initialized,
            EscrowError::EscrowNotActive
        );
        require!(
            ctx.accounts.maker.key() == escrow.maker,
            EscrowError::UnauthorizedCancellation
        );

        // If active, return tokens from vault to maker
        if escrow.state == EscrowState::Active {
            let escrow_key = ctx.accounts.escrow.key();
            let seeds = &[
                b"escrow",
                escrow_key.as_ref(),
                &[escrow.bump],
            ];
            let signer_seeds = &[&seeds[..]];

            let cpi_accounts = Transfer {
                from: ctx.accounts.vault.to_account_info(),
                to: ctx.accounts.maker_token_a.to_account_info(),
                authority: ctx.accounts.escrow.to_account_info(),
            };
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer_seeds,
            );
            token::transfer(cpi_ctx, escrow.amount_a)?;
        }

        let escrow = &mut ctx.accounts.escrow;
        escrow.state = EscrowState::Cancelled;

        emit!(EscrowCancelled {
            escrow: ctx.accounts.escrow.key(),
            maker: escrow.maker,
        });

        Ok(())
    }

    /// Close expired escrow and return tokens
    pub fn expire(ctx: Context<Expire>) -> Result<()> {
        let escrow = &ctx.accounts.escrow;
        
        require!(escrow.state == EscrowState::Active, EscrowError::EscrowNotActive);
        
        let clock = Clock::get()?;
        require!(clock.unix_timestamp >= escrow.expiration, EscrowError::EscrowNotExpired);

        // Return tokens to maker
        let escrow_key = ctx.accounts.escrow.key();
        let seeds = &[
            b"escrow",
            escrow_key.as_ref(),
            &[escrow.bump],
        ];
        let signer_seeds = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.vault.to_account_info(),
            to: ctx.accounts.maker_token_a.to_account_info(),
            authority: ctx.accounts.escrow.to_account_info(),
        };
        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts,
            signer_seeds,
        );
        token::transfer(cpi_ctx, escrow.amount_a)?;

        let escrow = &mut ctx.accounts.escrow;
        escrow.state = EscrowState::Expired;

        emit!(EscrowExpired {
            escrow: ctx.accounts.escrow.key(),
            maker: escrow.maker,
        });

        Ok(())
    }
}

// === Account Contexts ===

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,

    #[account(
        init,
        payer = maker,
        space = 8 + Escrow::INIT_SPACE,
        seeds = [b"escrow", maker.key().as_ref()],
        bump
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        init,
        payer = maker,
        token::mint = mint_a,
        token::authority = escrow,
    )]
    pub vault: Account<'info, TokenAccount>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        mut,
        constraint = escrow.maker == maker.key(),
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        constraint = maker_token_a.mint == escrow.mint_a,
        constraint = maker_token_a.owner == maker.key(),
    )]
    pub maker_token_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault.key() == escrow.vault,
    )]
    pub vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Exchange<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    #[account(mut)]
    pub escrow: Account<'info, Escrow>,

    /// CHECK: Maker account to receive token B
    pub maker: AccountInfo<'info>,

    #[account(
        mut,
        constraint = taker_token_a.mint == escrow.mint_a,
        constraint = taker_token_a.owner == taker.key(),
    )]
    pub taker_token_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = taker_token_b.mint == escrow.mint_b,
        constraint = taker_token_b.owner == taker.key(),
    )]
    pub taker_token_b: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = maker_token_b.mint == escrow.mint_b,
        constraint = maker_token_b.owner == escrow.maker,
    )]
    pub maker_token_b: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault.key() == escrow.vault,
    )]
    pub vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Cancel<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(
        mut,
        constraint = escrow.maker == maker.key(),
    )]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        constraint = maker_token_a.mint == escrow.mint_a,
        constraint = maker_token_a.owner == maker.key(),
    )]
    pub maker_token_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault.key() == escrow.vault,
    )]
    pub vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct Expire<'info> {
    #[account(mut)]
    pub escrow: Account<'info, Escrow>,

    #[account(
        mut,
        constraint = maker_token_a.mint == escrow.mint_a,
        constraint = maker_token_a.owner == escrow.maker,
    )]
    pub maker_token_a: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = vault.key() == escrow.vault,
    )]
    pub vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

// === State ===

#[account]
#[derive(InitSpace)]
pub struct Escrow {
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub vault: Pubkey,
    pub expiration: i64,
    pub state: EscrowState,
    pub bump: u8,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, InitSpace)]
pub enum EscrowState {
    Initialized,
    Active,
    Completed,
    Cancelled,
    Expired,
}

// === Errors ===

#[error_code]
pub enum EscrowError {
    #[msg("Amount must be greater than 0")]
    InvalidAmount,
    #[msg("Expiration must be in the future")]
    InvalidExpiration,
    #[msg("Escrow is not in active state")]
    EscrowNotActive,
    #[msg("Escrow has expired")]
    EscrowExpired,
    #[msg("Escrow has not expired yet")]
    EscrowNotExpired,
    #[msg("Only maker can cancel the escrow")]
    UnauthorizedCancellation,
    #[msg("Insufficient token balance")]
    InsufficientFunds,
}

// === Events ===

#[event]
pub struct EscrowDeposited {
    pub escrow: Pubkey,
    pub maker: Pubkey,
    pub amount: u64,
}

#[event]
pub struct EscrowCompleted {
    pub escrow: Pubkey,
    pub maker: Pubkey,
    pub taker: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
}

#[event]
pub struct EscrowCancelled {
    pub escrow: Pubkey,
    pub maker: Pubkey,
}

#[event]
pub struct EscrowExpired {
    pub escrow: Pubkey,
    pub maker: Pubkey,
}
