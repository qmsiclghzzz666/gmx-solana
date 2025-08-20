use anchor_lang::prelude::*;
use gmsol_model::utils::apply_factor;
use gmsol_programs::gmsol_store::constants::{MARKET_DECIMALS, MARKET_USD_UNIT};
use gmsol_programs::gmsol_store::{
    accounts::{Store, UserHeader},
    cpi as gt_cpi,
    cpi::accounts::{MintGtReward as GtMintCtx, UpdateGtCumulativeInvCostFactor as GtUpdateCtx},
    cpi::Return as GtReturn,
    program::GmsolStore,
};

const SECONDS_PER_YEAR: u128 = 31_557_600; // 365.25 * 24 * 3600
const PERCENT_DIVISOR: u64 = 100;
const GT_APY_PER_SEC: u128 =
    15u128 * MARKET_USD_UNIT as u128 / PERCENT_DIVISOR as u128 / SECONDS_PER_YEAR as u128;

declare_id!("BGDJg2u2NWwUE5q4Q4masGCFBVAhJ5pKrMbVSwjVwo8m");

#[program]
pub mod gmsol_liquidity_provider {
    use super::*;

    /// Initialize LP staking program
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        global_state.authority = ctx.accounts.authority.key();
        global_state.pending_authority = Pubkey::default();
        global_state.gt_mint = ctx.accounts.gt_mint.key();
        // APY per-second (scaled by 1e20): 15% APR -> (15 * 1e20) / (100 * SECONDS_PER_YEAR)
        global_state.gt_apy_per_sec = GT_APY_PER_SEC;
        global_state.lp_token_price = MARKET_USD_UNIT; // $1.00 in 1e20 units
        global_state.bump = ctx.bumps.global_state;
        msg!("LP staking program initialized, GT APY: 15%, GT price: $1.00");
        Ok(())
    }

    /// Create a new LP position and snapshot stake-time values
    pub fn stake_lp(
        ctx: Context<StakeLp>,
        position_id: u64,
        lp_staked_amount: u64,
        lp_staked_value: u128, // scaled USD at stake time
    ) -> Result<()> {
        let now = Clock::get()?.unix_timestamp;

        // Use GlobalState PDA as controller for GT CPI
        let gs_seeds: &[&[u8]] = &[b"global_state", &[ctx.accounts.global_state.bump]];
        let signer_seeds: &[&[&[u8]]] = &[gs_seeds];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.gt_program.to_account_info(),
            GtUpdateCtx {
                authority: ctx.accounts.global_state.to_account_info(),
                store: ctx.accounts.gt_store.to_account_info(),
            },
            signer_seeds,
        );
        // Snapshot C(start) at stake time
        let r: GtReturn<u128> = gt_cpi::update_gt_cumulative_inv_cost_factor(cpi_ctx)?;
        let c_start: u128 = r.get();

        // Init position fields
        let position = &mut ctx.accounts.position;
        position.owner = ctx.accounts.owner.key();
        position.global_state = ctx.accounts.global_state.key();
        position.position_id = position_id;
        position.staked_amount = lp_staked_amount;
        position.staked_value_usd = lp_staked_value;
        position.stake_start_time = now;
        position.cum_inv_cost = c_start;
        position.bump = ctx.bumps.position;

        msg!(
            "Stake created: owner={}, amount={}, value(1e20)={}, start_ts={}, C_start={}, pos_id={}",
            position.owner,
            lp_staked_amount,
            lp_staked_value,
            now,
            c_start,
            position_id
        );
        Ok(())
    }

    /// Calculate GT rewards for LP based on stored Position data
    pub fn calculate_gt_reward(ctx: Context<CalculateGtReward>) -> Result<()> {
        let global_state = &ctx.accounts.global_state;
        let position = &ctx.accounts.position;
        let current_time = Clock::get()?.unix_timestamp;

        // duration in seconds (non-negative)
        let duration_seconds = current_time.saturating_sub(position.stake_start_time);

        // --- Update GT cumulative inverse cost factor via CPI and read current cumulative value ---
        // Use the GlobalState PDA as GT_CONTROLLER authority
        let gs_seeds: &[&[u8]] = &[b"global_state", &[ctx.accounts.global_state.bump]];
        let signer_seeds: &[&[&[u8]]] = &[gs_seeds];

        let cpi_ctx = CpiContext::new_with_signer(
            ctx.accounts.gt_program.to_account_info(),
            GtUpdateCtx {
                authority: ctx.accounts.global_state.to_account_info(),
                store: ctx.accounts.gt_store.to_account_info(),
            },
            signer_seeds,
        );

        let r: GtReturn<u128> = gt_cpi::update_gt_cumulative_inv_cost_factor(cpi_ctx)?;
        let cum_now: u128 = r.get();

        // Integral over [start, now] (C(now) - C(start)); require monotonicity to avoid masking bugs
        require!(cum_now >= position.cum_inv_cost, ErrorCode::InvalidArgument);
        let inv_cost_integral = cum_now - position.cum_inv_cost;

        msg!(
            "GT inverse-cost cumulative: start={}, now={}, integral={}",
            position.cum_inv_cost,
            cum_now,
            inv_cost_integral
        );

        // Use the staked USD value captured at stake time (already 1e20 scaled)
        let staked_value_usd = position.staked_value_usd;

        // Read GT decimals from GT store and derive GT amount unit (10^decimals)
        let gt_decimals_u8 = {
            let store = ctx.accounts.gt_store.load()?;
            store.gt.decimals
        };
        let gt_decimals: u32 = gt_decimals_u8 as u32;
        let gt_amount_unit: u128 = 10u128.pow(gt_decimals);

        // Calculate GT reward amount in raw base units
        let gt_reward_raw = calculate_gt_reward_amount(
            staked_value_usd,
            duration_seconds,
            global_state.gt_apy_per_sec,
            inv_cost_integral,
        )?;

        // For display, also compute human-readable GT (floored)
        let gt_whole = (gt_reward_raw / (gt_amount_unit as u64)) as u64;

        msg!("Staked amount (raw): {}", position.staked_amount);
        msg!("Staked value (USD, 1e20): {}", staked_value_usd);
        msg!("Staking duration (s): {}", duration_seconds);
        msg!("GT APY per-second (1e20): {}", global_state.gt_apy_per_sec);
        msg!(
            "Calculated GT reward (raw, decimals={}): {}",
            gt_decimals,
            gt_reward_raw
        );
        msg!("Calculated GT reward (whole GT, floored): {} GT", gt_whole);
        Ok(())
    }

    /// Claim GT rewards for a position, minting tokens and updating snapshot
    pub fn claim_gt(ctx: Context<ClaimGt>, _position_id: u64) -> Result<()> {
        let position = &mut ctx.accounts.position;
        let global_state = &ctx.accounts.global_state;

        // Use GlobalState PDA as controller for GT CPI
        let gs_seeds: &[&[u8]] = &[b"global_state", &[global_state.bump]];
        let signer_seeds: &[&[&[u8]]] = &[gs_seeds];

        let update_ctx = CpiContext::new_with_signer(
            ctx.accounts.gt_program.to_account_info(),
            GtUpdateCtx {
                authority: global_state.to_account_info(),
                store: ctx.accounts.store.to_account_info(),
            },
            signer_seeds,
        );

        // 1) Update GT cumulative inverse cost factor and get current cumulative value
        let r: GtReturn<u128> = gt_cpi::update_gt_cumulative_inv_cost_factor(update_ctx)?;
        let cum_now: u128 = r.get();
        let prev_cum: u128 = position.cum_inv_cost;

        // 2) Compute integral over [last_snapshot, now]
        require!(cum_now >= prev_cum, ErrorCode::InvalidArgument);
        let inv_cost_integral = cum_now - prev_cum;

        // 3) Calculate GT reward amount
        let current_time = Clock::get()?.unix_timestamp;
        let duration_seconds = current_time.saturating_sub(position.stake_start_time);
        let gt_reward_raw = calculate_gt_reward_amount(
            position.staked_value_usd,
            duration_seconds,
            global_state.gt_apy_per_sec,
            inv_cost_integral,
        )?;

        // 4) Mint GT tokens to the user's GT account (authority = GlobalState PDA)
        if gt_reward_raw > 0 {
            let mint_ctx = CpiContext::new_with_signer(
                ctx.accounts.gt_program.to_account_info(),
                GtMintCtx {
                    authority: global_state.to_account_info(),
                    store: ctx.accounts.store.to_account_info(),
                    user: ctx.accounts.gt_user.to_account_info(),
                    event_authority: ctx.accounts.event_authority.to_account_info(),
                    program: ctx.accounts.gt_program.to_account_info(),
                },
                signer_seeds,
            );
            gt_cpi::mint_gt_reward(mint_ctx, gt_reward_raw)?;
        }

        // 5) Update snapshot to now for future claims (do NOT change stake_start_time)
        position.cum_inv_cost = cum_now;

        msg!(
            "Claimed GT: amount_raw={} | pos_id={} | duration={}s | C_prev->C_now: {}->{}, integral={}",
            gt_reward_raw,
            position.position_id,
            duration_seconds,
            prev_cum,
            cum_now,
            inv_cost_integral
        );

        Ok(())
    }

    /// Update GT APY parameter
    pub fn update_gt_apy_per_sec(
        ctx: Context<UpdateGtApy>,
        new_apy_per_sec: u128, // scaled by 1e20
    ) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        global_state.gt_apy_per_sec = new_apy_per_sec;
        msg!("GT APY per-second (1e20) updated to: {}", new_apy_per_sec);
        Ok(())
    }

    /// Propose transferring program authority to `new_authority` (two-step handover).
    pub fn transfer_authority(
        ctx: Context<TransferAuthority>,
        new_authority: Pubkey,
    ) -> Result<()> {
        let gs = &mut ctx.accounts.global_state;
        require!(
            new_authority != Pubkey::default(),
            ErrorCode::InvalidArgument
        );
        gs.pending_authority = new_authority;
        msg!(
            "Authority transfer proposed: pending_authority = {}",
            new_authority
        );
        Ok(())
    }

    /// Accept authority if you are the pending_authority; finalizes the handover.
    pub fn accept_authority(ctx: Context<AcceptAuthority>) -> Result<()> {
        let gs = &mut ctx.accounts.global_state;
        gs.authority = ctx.accounts.pending_authority.key();
        gs.pending_authority = Pubkey::default();
        msg!(
            "Authority transfer accepted: new authority = {}",
            gs.authority
        );
        Ok(())
    }
}

/// Calculate GT reward amount (returns raw amount in base units, respecting token decimals)
/// Expects the integral over the window: inv_cost_integral = C(now) - C(start), where
/// C(t) accumulates (MARKET_USD_UNIT / price(t)) dt. No extra multiply by `duration_seconds`
/// is required because time is already integrated inside `inv_cost_integral`.
fn calculate_gt_reward_amount(
    staked_value_usd: u128,  // Already scaled USD value (e.g., in MARKET_USD_UNIT)
    duration_seconds: i64,   // current_time - stake_start_time
    gt_apy_per_sec: u128,    // per-second factor scaled by 1e20
    inv_cost_integral: u128, // ∫ (MARKET_USD_UNIT / price(t)) dt over the interval [start, now]
) -> Result<u64> {
    require!(duration_seconds >= 0, ErrorCode::Unauthorized);

    // Convert notional USD to per-second notionals using APY per second
    let per_sec_factor = apply_factor::<u128, MARKET_DECIMALS>(&staked_value_usd, &gt_apy_per_sec)
        .ok_or(ErrorCode::MathOverflow)?;

    // Apply the integral of MARKET_USD_UNIT / price(t) over time.
    let gt_raw = apply_factor::<u128, MARKET_DECIMALS>(&per_sec_factor, &inv_cost_integral)
        .ok_or(ErrorCode::MathOverflow)?;

    Ok(gt_raw.min(u64::MAX as u128) as u64)
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + GlobalState::INIT_SPACE,
        seeds = [b"global_state"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: GT token mint address
    pub gt_mint: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

/// Accounts context for staking LP tokens and creating a Position
#[derive(Accounts)]
#[instruction(position_id: u64)]
pub struct StakeLp<'info> {
    /// Global config (PDA)
    #[account(seeds = [b"global_state"], bump = global_state.bump)]
    pub global_state: Account<'info, GlobalState>,
    /// Position PDA to initialize for (global_state, owner, position_id)
    #[account(
        init,
        payer = owner,
        space = 8 + Position::INIT_SPACE,
        seeds = [
            b"position",
            global_state.key().as_ref(),
            owner.key().as_ref(),
            &position_id.to_le_bytes(),
        ],
        bump
    )]
    pub position: Account<'info, Position>,
    /// The GT Store account (mutated by CPI)
    #[account(mut)]
    pub gt_store: AccountLoader<'info, Store>,
    /// GT program
    pub gt_program: Program<'info, GmsolStore>,
    /// Owner paying rent and recorded as position owner
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Accounts context for calculating GT reward from a Position
#[derive(Accounts)]
#[instruction(position_id: u64)]
pub struct CalculateGtReward<'info> {
    /// Global config (PDA)
    #[account(seeds = [b"global_state"], bump = global_state.bump)]
    pub global_state: Account<'info, GlobalState>,
    /// The GT Store account (loaded & mutated by CPI)
    #[account(mut)]
    pub gt_store: AccountLoader<'info, Store>,
    /// The GT program
    pub gt_program: Program<'info, GmsolStore>,
    /// Position tied to (global_state, owner, position_id)
    #[account(
        seeds = [
            b"position",
            global_state.key().as_ref(),
            owner.key().as_ref(),
            &position_id.to_le_bytes(),
        ],
        bump = position.bump,
        has_one = owner,
        has_one = global_state
    )]
    pub position: Account<'info, Position>,
    /// Owner of the position (not required to sign for read-only calc)
    /// CHECK: only used for PDA seeds match via has_one
    pub owner: UncheckedAccount<'info>,
}

/// Accounts context for claiming GT reward and updating Position snapshot
#[derive(Accounts)]
#[instruction(position_id: u64)]
pub struct ClaimGt<'info> {
    /// Global config (PDA)
    #[account(seeds = [b"global_state"], bump = global_state.bump)]
    pub global_state: Account<'info, GlobalState>,

    /// The GT Store account (mutated by CPI)
    #[account(mut)]
    pub store: AccountLoader<'info, Store>,

    /// The GT program
    pub gt_program: Program<'info, GmsolStore>,

    /// Position tied to (global_state, owner, position_id)
    #[account(
        mut,
        seeds = [
            b"position",
            global_state.key().as_ref(),
            owner.key().as_ref(),
            &position_id.to_le_bytes(),
        ],
        bump = position.bump,
        has_one = owner,
        has_one = global_state
    )]
    pub position: Account<'info, Position>,

    /// Owner of the position
    pub owner: Signer<'info>,

    /// GT User account (mut) managed by the GT program; must correspond to (store, owner)
    #[account(
        mut,
        has_one = owner,
        has_one = store,
    )]
    pub gt_user: AccountLoader<'info, UserHeader>,

    /// CHECK: GT program's event authority PDA required by #[event_cpi] calls
    pub event_authority: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct UpdateGtApy<'info> {
    /// Global config (PDA). The `authority` signer must match `global_state.authority`.
    #[account(mut, seeds = [b"global_state"], bump = global_state.bump, has_one = authority)]
    pub global_state: Account<'info, GlobalState>,
    /// Current authority
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct TransferAuthority<'info> {
    /// Global config (PDA). The `authority` signer must match `global_state.authority`.
    #[account(mut, seeds = [b"global_state"], bump = global_state.bump, has_one = authority)]
    pub global_state: Account<'info, GlobalState>,
    /// Current authority proposing a transfer
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct AcceptAuthority<'info> {
    /// Global config (PDA). The signer must equal `global_state.pending_authority`.
    #[account(mut, seeds = [b"global_state"], bump = global_state.bump, has_one = pending_authority)]
    pub global_state: Account<'info, GlobalState>,
    /// Pending authority accepting control (must match `global_state.pending_authority`)
    pub pending_authority: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct GlobalState {
    /// Program administrator with governance privileges
    pub authority: Pubkey,
    /// Pending authority awaiting acceptance (Pubkey::default() if none)
    pub pending_authority: Pubkey,
    /// GT token mint address
    pub gt_mint: Pubkey,
    /// Per-second APY factor scaled by 1e20 (MARKET_USD_UNIT).
    /// Example: for 15% APR, set to (15 * 1e20) / (100 * SECONDS_PER_YEAR).
    pub gt_apy_per_sec: u128,
    /// LP token price in USD scaled by 1e20
    pub lp_token_price: u128,
    /// PDA bump for this GlobalState (derived from seed ["global_state"])
    pub bump: u8,
}

/// Position account to persist LP stake data and snapshot stake-time values
#[account]
#[derive(InitSpace)]
pub struct Position {
    /// Owner of this LP position
    pub owner: Pubkey,
    /// Ties position to a specific GlobalState
    pub global_state: Pubkey,
    /// Position id to allow multiple positions per owner
    pub position_id: u64,
    /// Staked LP amount at stake time (raw amount as provided by caller; optional semantics)
    pub staked_amount: u64,
    /// Staked value in USD (scaled by 1e20) captured at stake time
    pub staked_value_usd: u128,
    /// Stake start unix timestamp (seconds)
    pub stake_start_time: i64,
    /// Cumulative inverse-cost factor snapshot (last claim/stake checkpoint)
    pub cum_inv_cost: u128,
    /// PDA bump
    pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized operation")]
    Unauthorized,
    #[msg("Invalid argument")]
    InvalidArgument,
    #[msg("Math overflow")]
    MathOverflow,
}
