const SECONDS_PER_YEAR: u128 = 31_557_600; // 365.25 * 24 * 3600
const GT_DECIMALS: u32 = 7; // GT token decimals
const GT_AMOUNT_UNIT: u128 = 10u128.pow(GT_DECIMALS);
const MARKET_USD_UNIT: u128 = 10u128.pow(20);
use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod gmsol_liquidity_provider {
    use super::*;

    /// Initialize LP staking program
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        global_state.authority = ctx.accounts.authority.key();
        global_state.gt_mint = ctx.accounts.gt_mint.key();
        // APY as whole percent, price as 1e20
        global_state.gt_apy = 15; // 15% APY (unscaled percentage)
        global_state.gt_price_usd = MARKET_USD_UNIT; // $1.00 in 1e20 units
        msg!("LP staking program initialized, GT APY: 15%, GT price: $1.00");
        Ok(())
    }

    /// Calculate GT rewards for LP
    pub fn calculate_gt_reward(
        ctx: Context<CalculateGtReward>,
        lp_staked_amount: u64,
        stake_start_time: i64,
    ) -> Result<()> {
        let global_state = &ctx.accounts.global_state;
        let current_time = Clock::get()?.unix_timestamp;

        // duration in seconds (non-negative)
        let duration_seconds = current_time.saturating_sub(stake_start_time);

        // In this version, treat lp_staked_amount as raw amount, convert to scaled USD value
        let staked_value_usd = (lp_staked_amount as u128).saturating_mul(MARKET_USD_UNIT);

        // Calculate GT reward amount in raw base units (decimals = 7)
        let gt_reward_raw = calculate_gt_reward_amount_int(
            staked_value_usd,
            duration_seconds,
            global_state.gt_apy as u128,
            global_state.gt_price_usd,
        )?;

        // For display, also compute human-readable GT (floored)
        let gt_whole = (gt_reward_raw / (GT_AMOUNT_UNIT as u64)) as u64;

        msg!("Staked amount (scaled USD): {}", staked_value_usd);
        msg!("Staking duration (s): {}", duration_seconds);
        msg!("GT APY (whole %): {}", global_state.gt_apy);
        msg!("GT price USD (1e20 units): {}", global_state.gt_price_usd);
        msg!(
            "Calculated GT reward (raw, decimals={}): {}",
            GT_DECIMALS,
            gt_reward_raw
        );
        msg!("Calculated GT reward (whole GT, floored): {} GT", gt_whole);
        Ok(())
    }

    /// Update GT APY parameter
    pub fn update_gt_apy(
        ctx: Context<UpdateGtApY>,
        new_apy: u64, // APY as whole percent, e.g., 15 for 15%
    ) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        require!(
            ctx.accounts.authority.key() == global_state.authority,
            ErrorCode::Unauthorized
        );
        global_state.gt_apy = new_apy;
        msg!("GT APY (whole %) updated to: {}", new_apy);
        Ok(())
    }

    /// Update GT price parameter
    pub fn update_gt_price(
        ctx: Context<UpdateGtPrice>,
        new_price_usd: u128, // Price scaled by 1e20, e.g., $1.00 -> 10^20
    ) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        require!(
            ctx.accounts.authority.key() == global_state.authority,
            ErrorCode::Unauthorized
        );
        global_state.gt_price_usd = new_price_usd;
        msg!("GT price USD (1e20 units) updated to: {}", new_price_usd);
        Ok(())
    }
}

/// Calculate GT reward amount (returns raw amount in base units, respecting token decimals)
fn calculate_gt_reward_amount_int(
    staked_value_usd: u128, // Already scaled USD value (e.g., in MARKET_USD_UNIT)
    duration_seconds: i64,  // current_time - stake_start_time
    gt_apy: u128,           // APY as whole percent (e.g., 15 for 15%)
    gt_price_usd: u128,     // USD price scaled by MARKET_USD_UNIT (1e20)
) -> Result<u64> {
    require!(duration_seconds >= 0, ErrorCode::Unauthorized);

    let dur = duration_seconds as u128;

    // interest_usd_scaled = staked * apy% * duration / (100 * SECONDS_PER_YEAR)
    let mut acc = staked_value_usd;
    acc = acc.saturating_mul(gt_apy);
    acc = acc.saturating_div(100);
    acc = acc.saturating_mul(dur);
    acc = acc.saturating_div(SECONDS_PER_YEAR);

    if gt_price_usd == 0 {
        return Ok(0);
    }

    // Convert USD (1e20) to GT raw units (1e7):
    // gt_raw = interest_usd_scaled * GT_AMOUNT_UNIT / gt_price_usd
    let gt_raw = acc
        .saturating_mul(GT_AMOUNT_UNIT)
        .saturating_div(gt_price_usd);

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

#[derive(Accounts)]
pub struct CalculateGtReward<'info> {
    pub global_state: Account<'info, GlobalState>,
}

#[derive(Accounts)]
pub struct UpdateGtApY<'info> {
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,

    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateGtPrice<'info> {
    #[account(mut)]
    pub global_state: Account<'info, GlobalState>,

    pub authority: Signer<'info>,
}

#[account]
#[derive(InitSpace)]
pub struct GlobalState {
    pub authority: Pubkey,  // Program administrator
    pub gt_mint: Pubkey,    // GT token mint address
    pub gt_apy: u64,        // APY as whole percent (e.g., 15 for 15%)
    pub gt_price_usd: u128, // USD price scaled by 1e20
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized operation")]
    Unauthorized,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_gt_reward_amount_int() {
        // Case 1: $1000, 1 year, 15% APY, $1.00
        let staked_value = 1000u64;
        let duration_seconds = SECONDS_PER_YEAR as i64;
        let apy = 15u128; // 15%
        let price = MARKET_USD_UNIT; // $1.00 * 1e20

        // Multiply staked_value by MARKET_USD_UNIT to scale
        let staked_value_scaled = (staked_value as u128).saturating_mul(MARKET_USD_UNIT);

        let gt_reward_raw =
            calculate_gt_reward_amount_int(staked_value_scaled, duration_seconds, apy, price)
                .unwrap();

        // Expected: 1000 * 0.15 = 150 GT -> raw = 150 * 10^7
        assert_eq!(gt_reward_raw, 150 * (GT_AMOUNT_UNIT as u64));

        // Case 2: $2000, 0.5 years, 15% APY, $1.00
        let staked_value = 2000u64;
        let duration_seconds = (SECONDS_PER_YEAR / 2) as i64;

        let staked_value_scaled = (staked_value as u128).saturating_mul(MARKET_USD_UNIT);

        let gt_reward_raw =
            calculate_gt_reward_amount_int(staked_value_scaled, duration_seconds, apy, price)
                .unwrap();

        // Expected: 2000 * 0.15 * 0.5 = 150 GT
        assert_eq!(gt_reward_raw, 150 * (GT_AMOUNT_UNIT as u64));

        // Case 3: $1000, 1 year, 20% APY, $0.50
        let staked_value = 1000u64;
        let duration_seconds = SECONDS_PER_YEAR as i64;
        let apy = 20u128; // 20%
        let price = MARKET_USD_UNIT / 2; // $0.50 * 1e20

        let staked_value_scaled = (staked_value as u128).saturating_mul(MARKET_USD_UNIT);

        let gt_reward_raw =
            calculate_gt_reward_amount_int(staked_value_scaled, duration_seconds, apy, price)
                .unwrap();

        // Expected: 1000 * 0.20 / 0.50 = 400 GT
        assert_eq!(gt_reward_raw, 400 * (GT_AMOUNT_UNIT as u64));
    }

    #[test]
    fn test_edge_cases_int() {
        // Stake $0
        let r = calculate_gt_reward_amount_int(0, SECONDS_PER_YEAR as i64, 15, MARKET_USD_UNIT)
            .unwrap();
        assert_eq!(r, 0);

        // Duration 0
        let staked_value_scaled = 1000u128.saturating_mul(MARKET_USD_UNIT);
        let r =
            calculate_gt_reward_amount_int(staked_value_scaled, 0, 15, MARKET_USD_UNIT).unwrap();
        assert_eq!(r, 0);

        // Price 0 -> safe-guard returns 0
        let r = calculate_gt_reward_amount_int(staked_value_scaled, SECONDS_PER_YEAR as i64, 15, 0)
            .unwrap();
        assert_eq!(r, 0);
    }
}
