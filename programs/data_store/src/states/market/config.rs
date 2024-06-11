use anchor_lang::prelude::*;

use crate::{constants, states::Factor};

/// Market Config.
#[zero_copy]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct MarketConfig {
    // Swap impact.
    pub(super) swap_imapct_exponent: Factor,
    pub(super) swap_impact_positive_factor: Factor,
    pub(super) swap_impact_negative_factor: Factor,
    // Swap fee.
    pub(super) swap_fee_receiver_factor: Factor,
    pub(super) swap_fee_factor_for_positive_impact: Factor,
    pub(super) swap_fee_factor_for_negative_impact: Factor,
    // Position general.
    pub(super) min_position_size_usd: Factor,
    pub(super) min_collateral_value: Factor,
    pub(super) min_collateral_factor: Factor,
    pub(super) max_positive_position_impact_factor: Factor,
    pub(super) max_negative_position_impact_factor: Factor,
    pub(super) max_position_impact_factor_for_liquidations: Factor,
    // Position impact.
    pub(super) position_impact_exponent: Factor,
    pub(super) position_impact_positive_factor: Factor,
    pub(super) position_impact_negative_factor: Factor,
    // Order fee.
    pub(super) order_fee_receiver_factor: Factor,
    pub(super) order_fee_factor_for_positive_impact: Factor,
    pub(super) order_fee_factor_for_negative_impact: Factor,
    // Position impact distribtuion.
    pub(super) position_impact_distribute_factor: Factor,
    pub(super) min_position_impact_pool_amount: Factor,
    // Borrowing fee.
    pub(super) borrowing_fee_receiver_factor: Factor,
    pub(super) borrowing_fee_factor_for_long: Factor,
    pub(super) borrowing_fee_factor_for_short: Factor,
    pub(super) borrowing_fee_exponent_for_long: Factor,
    pub(super) borrowing_fee_exponent_for_short: Factor,
    // Funding fee.
    pub(super) funding_fee_exponent: Factor,
    pub(super) funding_fee_factor: Factor,
    pub(super) funding_fee_max_factor_per_second: Factor,
    pub(super) funding_fee_min_factor_per_second: Factor,
    pub(super) funding_fee_increase_factor_per_second: Factor,
    pub(super) funding_fee_decrease_factor_per_second: Factor,
    pub(super) funding_fee_threshold_for_stable_funding: Factor,
    pub(super) funding_fee_threshold_for_decrease_funding: Factor,
    // Reserve factor.
    pub(super) reserve_factor: Factor,
    pub(super) open_interest_reserve_factor: Factor,
    // Max pnl factor.
    pub(super) max_pnl_factor_for_long_deposit: Factor,
    pub(super) max_pnl_factor_for_short_deposit: Factor,
    pub(super) max_pnl_factor_for_long_withdrawal: Factor,
    pub(super) max_pnl_factor_for_short_withdrawal: Factor,
    // Other boundary.
    pub(super) max_pool_amount_for_long_token: Factor,
    pub(super) max_pool_amount_for_short_token: Factor,
    pub(super) max_pool_value_for_deposit_for_long_token: Factor,
    pub(super) max_pool_value_for_deposit_for_short_token: Factor,
    pub(super) max_open_interest_for_long: Factor,
    pub(super) max_open_interest_for_short: Factor,
    reserved: [Factor; 19],
}

impl MarketConfig {
    pub(super) fn init(&mut self) {
        self.swap_imapct_exponent = constants::DEFAULT_SWAP_IMPACT_EXPONENT;
        self.swap_impact_positive_factor = constants::DEFAULT_SWAP_IMPACT_POSITIVE_FACTOR;
        self.swap_impact_positive_factor = constants::DEFAULT_SWAP_IMPACT_NEGATIVE_FACTOR;

        self.swap_fee_receiver_factor = constants::DEFAULT_RECEIVER_FACTOR;
        self.swap_fee_factor_for_positive_impact =
            constants::DEFAULT_SWAP_FEE_FACTOR_FOR_POSITIVE_IMPACT;
        self.swap_fee_factor_for_negative_impact =
            constants::DEFAULT_SWAP_FEE_FACTOR_FOR_NEGATIVE_IMPACT;

        self.min_position_size_usd = constants::DEFAULT_MIN_POSITION_SIZE_USD;
        self.min_collateral_value = constants::DEFAULT_MIN_COLLATERAL_VALUE;
        self.min_collateral_factor = constants::DEFAULT_MIN_COLLATERAL_FACTOR;
        self.max_positive_position_impact_factor =
            constants::DEFAULT_MAX_POSITIVE_POSITION_IMPACT_FACTOR;
        self.max_negative_position_impact_factor =
            constants::DEFAULT_MAX_NEGATIVE_POSITION_IMPACT_FACTOR;
        self.max_position_impact_factor_for_liquidations =
            constants::DEFAULT_MAX_POSITION_IMPACT_FACTOR_FOR_LIQUIDATIONS;

        self.position_impact_exponent = constants::DEFAULT_POSITION_IMPACT_EXPONENT;
        self.position_impact_positive_factor = constants::DEFAULT_POSITION_IMPACT_POSITIVE_FACTOR;
        self.position_impact_negative_factor = constants::DEFAULT_POSITION_IMPACT_NEGATIVE_FACTOR;

        self.order_fee_receiver_factor = constants::DEFAULT_RECEIVER_FACTOR;
        self.order_fee_factor_for_positive_impact =
            constants::DEFAULT_ORDER_FEE_FACTOR_FOR_POSITIVE_IMPACT;
        self.order_fee_factor_for_negative_impact =
            constants::DEFAULT_ORDER_FEE_FACTOR_FOR_NEGATIVE_IMPACT;

        self.position_impact_distribute_factor =
            constants::DEFAULT_POSITION_IMPACT_DISTRIBUTE_FACTOR;
        self.min_position_impact_pool_amount = constants::DEFAULT_MIN_POSITION_IMPACT_POOL_AMOUNT;

        self.borrowing_fee_receiver_factor = constants::DEFAULT_RECEIVER_FACTOR;
        self.borrowing_fee_factor_for_long = constants::DEFAULT_BORROWING_FEE_FACTOR_FOR_LONG;
        self.borrowing_fee_factor_for_short = constants::DEFAULT_BORROWING_FEE_FACTOR_FOR_SHORT;
        self.borrowing_fee_exponent_for_long = constants::DEFAULT_BORROWING_FEE_EXPONENT_FOR_LONG;
        self.borrowing_fee_exponent_for_short = constants::DEFAULT_BORROWING_FEE_EXPONENT_FOR_SHORT;

        self.funding_fee_exponent = constants::DEFAULT_FUNDING_FEE_EXPONENT;
        self.funding_fee_factor = constants::DEFAULT_FUNDING_FEE_FACTOR;
        self.funding_fee_max_factor_per_second =
            constants::DEFAULT_FUNDING_FEE_MAX_FACTOR_PER_SECOND;
        self.funding_fee_min_factor_per_second =
            constants::DEFAULT_FUNDING_FEE_MIN_FACTOR_PER_SECOND;
        self.funding_fee_increase_factor_per_second =
            constants::DEFAULT_FUNDING_FEE_INCREASE_FACTOR_PER_SECOND;
        self.funding_fee_decrease_factor_per_second =
            constants::DEFAULT_FUNDING_FEE_DECREASE_FACTOR_PER_SECOND;
        self.funding_fee_threshold_for_stable_funding =
            constants::DEFAULT_FUNDING_FEE_THRESHOLD_FOR_STABLE_FUNDING;
        self.funding_fee_threshold_for_decrease_funding =
            constants::DEFAULT_FUNDING_FEE_THRESHOLD_FOR_DECREASE_FUNDING;

        self.reserve_factor = constants::DEFAULT_RECEIVER_FACTOR;
        self.open_interest_reserve_factor = constants::DEFAULT_OPEN_INTEREST_RESERVE_FACTOR;

        self.max_pnl_factor_for_long_deposit = constants::DEFAULT_MAX_PNL_FACTOR_FOR_LONG_DEPOSIT;
        self.max_pnl_factor_for_short_deposit = constants::DEFAULT_MAX_PNL_FACTOR_FOR_SHORT_DEPOSIT;
        self.max_pnl_factor_for_long_withdrawal =
            constants::DEFAULT_MAX_PNL_FACTOR_FOR_LONG_WITHDRAWAL;
        self.max_pnl_factor_for_short_withdrawal =
            constants::DEFAULT_MAX_PNL_FACTOR_FOR_SHORT_WITHDRAWAL;

        self.max_pool_amount_for_long_token = constants::DEFAULT_MAX_POOL_AMOUNT_FOR_LONG_TOKEN;
        self.max_pool_amount_for_short_token = constants::DEFAULT_MAX_POOL_AMOUNT_FOR_SHORT_TOKEN;

        self.max_pool_value_for_deposit_for_long_token =
            constants::DEFAULT_MAX_POOL_VALUE_FOR_DEPOSIT_LONG_TOKEN;
        self.max_pool_value_for_deposit_for_short_token =
            constants::DEFAULT_MAX_POOL_VALUE_FOR_DEPOSIT_SHORT_TOKEN;

        self.max_open_interest_for_long = constants::DEFAULT_MAX_OPEN_INTEREST_FOR_LONG;
        self.max_open_interest_for_short = constants::DEFAULT_MAX_OPEN_INTEREST_FOR_SHORT;
    }
}
