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
        global_state.gt_apy = 0.15; // 15% APY (0.15 = 15.00%)
        global_state.gt_price_usd = 1000000; // GT price in micro USD (1.00 USD = 1,000,000 micro USD)
        
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
        
        // Calculate staking duration in years
        let duration_seconds = current_time.saturating_sub(stake_start_time);
        let duration_years = duration_seconds as f64 / (365.25 * 24.0 * 60.0 * 60.0); // Convert to years using 365.25 days
        
        // Calculate staked value (assuming 1 token = $1, in real applications should get price from oracle)
        let staked_value_usd = lp_staked_amount as f64;
        
        // Calculate GT reward amount
        let gt_reward = calculate_gt_reward_amount(
            staked_value_usd,
            duration_years,
            global_state.gt_apy,
            global_state.gt_price_usd,
        )?;
        
        msg!("Staked amount: {} tokens", lp_staked_amount);
        msg!("Staking duration: {:.2} years", duration_years);
        msg!("GT APY: {:.2}%", global_state.gt_apy * 100.0);
        msg!("GT price: ${:.6}", global_state.gt_price_usd as f64 / 1_000_000.0);
        msg!("Calculated GT reward: {} GT", gt_reward);
        
        Ok(())
    }

    /// Update GT APY parameter
    pub fn update_gt_apy(
        ctx: Context<UpdateGtApY>,
        new_apy: f64,
    ) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        require!(ctx.accounts.authority.key() == global_state.authority, ErrorCode::Unauthorized);
        
        global_state.gt_apy = new_apy;
        
        msg!("GT APY updated to: {:.2}%", new_apy * 100.0);
        Ok(())
    }

    /// Update GT price parameter
    pub fn update_gt_price(
        ctx: Context<UpdateGtPrice>,
        new_price_usd: u64,
    ) -> Result<()> {
        let global_state = &mut ctx.accounts.global_state;
        require!(ctx.accounts.authority.key() == global_state.authority, ErrorCode::Unauthorized);
        
        global_state.gt_price_usd = new_price_usd;
        
        msg!("GT price updated to: ${:.6}", new_price_usd as f64 / 1_000_000.0);
        Ok(())
    }
}

/// Calculate GT reward amount
fn calculate_gt_reward_amount(
    staked_value_usd: f64,
    duration_years: f64,
    gt_apy: f64,
    gt_price_usd_micro: u64,
) -> Result<u64> {
    // GT APY is already in decimal format (e.g., 0.15 = 15%)
    
    // Calculate annual return
    let annual_return_usd = staked_value_usd * gt_apy;
    
    // Calculate actual return based on staking duration
    let actual_return_usd = annual_return_usd * duration_years;
    
    // Convert GT price from micro USD to USD
    let gt_price_usd = gt_price_usd_micro as f64 / 1_000_000.0;
    
    // Calculate GT amount that can be purchased
    let gt_amount = actual_return_usd / gt_price_usd;
    
    // Convert to u64, floor down
    let gt_amount_u64 = gt_amount.floor() as u64;
    
    Ok(gt_amount_u64)
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
    pub authority: Pubkey,                    // Program administrator
    pub gt_mint: Pubkey,                      // GT token mint address
    pub gt_apy: f64,                          // GT reward APY (e.g., 0.15 = 15.00%)
    pub gt_price_usd: u64,                    // GT price (in micro USD, 1.00 USD = 1,000,000 micro USD)
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
    fn test_calculate_gt_reward_amount() {
        // Test case 1: Stake $1000, stake for 1 year, 15% APY, GT price $1.00
        let staked_value = 1000.0;
        let duration = 1.0;
        let apy = 0.15; // 15%
        let gt_price = 1_000_000; // $1.00
        
        let gt_reward = calculate_gt_reward_amount(staked_value, duration, apy, gt_price).unwrap();
        
        // Expected: 1000 * 0.15 = 150 GT
        assert_eq!(gt_reward, 150);
        
        // Test case 2: Stake $2000, stake for 0.5 years, 15% APY, GT price $1.00
        let staked_value = 2000.0;
        let duration = 0.5;
        let apy = 0.15; // 15%
        let gt_price = 1_000_000; // $1.00
        
        let gt_reward = calculate_gt_reward_amount(staked_value, duration, apy, gt_price).unwrap();
        
        // Expected: 2000 * 0.15 * 0.5 = 150 GT
        assert_eq!(gt_reward, 150);
        
        // Test case 3: Stake $1000, stake for 1 year, 20% APY, GT price $0.50
        let staked_value = 1000.0;
        let duration = 1.0;
        let apy = 0.20; // 20%
        let gt_price = 500_000; // $0.50
        
        let gt_reward = calculate_gt_reward_amount(staked_value, duration, apy, gt_price).unwrap();
        
        // Expected: 1000 * 0.20 / 0.50 = 400 GT
        assert_eq!(gt_reward, 400);
    }

    #[test]
    fn test_edge_cases() {
        // Test edge case: Stake $0
        let gt_reward = calculate_gt_reward_amount(0.0, 1.0, 0.15, 1_000_000).unwrap();
        assert_eq!(gt_reward, 0);
        
        // Test edge case: Staking duration 0 years
        let gt_reward = calculate_gt_reward_amount(1000.0, 0.0, 0.15, 1_000_000).unwrap();
        assert_eq!(gt_reward, 0);
        
        // Test edge case: APY is 0
        let gt_reward = calculate_gt_reward_amount(1000.0, 1.0, 0.0, 1_000_000).unwrap();
        assert_eq!(gt_reward, 0);
    }
}
