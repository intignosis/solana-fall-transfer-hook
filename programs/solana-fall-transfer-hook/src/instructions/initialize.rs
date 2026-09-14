use anchor_lang::prelude::*;
use anchor_spl::{token_2022, token_interface::Mint};

use crate::{ANCHOR_DISCRIMINATOR_SIZE, RateLimit, error::ErrorCode};

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    // Declared BEFORE rate_limit so the seeds below can reference it.
    pub mint: InterfaceAccount<'info, Mint>,
    #[account(
        init,
        payer = payer,
        // One rate limit per (mint, owner). A program-wide account would let any
        // single holder exhaust the cap and block every other holder.
        seeds = [b"rate_limit", mint.key().as_ref(), payer.key().as_ref()],
        bump,
        space = ANCHOR_DISCRIMINATOR_SIZE + RateLimit::INIT_SPACE,
    )]
    pub rate_limit: Account<'info, RateLimit>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    // The mint must belong to Token-2022. `InterfaceAccount<Mint>` accepts either
    // token program, so it proves this deserializes as a mint and not which program
    // owns it — the owner check is the part that matters, and it is ours to make.
    require_keys_eq!(
        *ctx.accounts.mint.to_account_info().owner,
        token_2022::ID,
        ErrorCode::InvalidMint
    );

    // Initialize the rate limit account with the authority, mint, max amount, and window start timestamp
    ctx.accounts.rate_limit.set_inner(RateLimit {
        authority: ctx.accounts.payer.key(),
        mint: ctx.accounts.mint.key(),
        max_amount: RateLimit::MAX_AMOUNT,
        window_start: Clock::get()?.unix_timestamp,
        amount_transferred: 0
    });

    Ok(())
}
