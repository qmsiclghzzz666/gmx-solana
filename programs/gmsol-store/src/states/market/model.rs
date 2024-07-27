use gmsol_model::PoolKind;

use crate::constants;

use super::{Market, Pool};

impl gmsol_model::BaseMarket<{ constants::MARKET_DECIMALS }> for Market {
    type Num = u128;

    type Signed = i128;

    type Pool = Pool;

    fn liquidity_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        self.try_pool(PoolKind::Primary)
    }

    fn claimable_fee_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        self.try_pool(PoolKind::ClaimableFee)
    }

    fn swap_impact_pool(&self) -> gmsol_model::Result<&Self::Pool> {
        self.try_pool(PoolKind::SwapImpact)
    }

    fn open_interest_pool(&self, is_long: bool) -> gmsol_model::Result<&Self::Pool> {
        self.try_pool(if is_long {
            PoolKind::OpenInterestForLong
        } else {
            PoolKind::OpenInterestForShort
        })
    }

    fn open_interest_in_tokens_pool(&self, is_long: bool) -> gmsol_model::Result<&Self::Pool> {
        self.try_pool(if is_long {
            PoolKind::OpenInterestInTokensForLong
        } else {
            PoolKind::OpenInterestInTokensForShort
        })
    }

    fn usd_to_amount_divisor(&self) -> Self::Num {
        constants::MARKET_USD_TO_AMOUNT_DIVISOR
    }

    fn max_pool_amount(&self, is_long_token: bool) -> gmsol_model::Result<Self::Num> {
        if is_long_token {
            Ok(self.config.max_pool_amount_for_long_token)
        } else {
            Ok(self.config.max_pool_amount_for_short_token)
        }
    }

    fn pnl_factor_config(
        &self,
        kind: gmsol_model::PnlFactorKind,
        is_long: bool,
    ) -> gmsol_model::Result<Self::Num> {
        use gmsol_model::PnlFactorKind;

        match (kind, is_long) {
            (PnlFactorKind::MaxAfterDeposit, true) => {
                Ok(self.config.max_pnl_factor_for_long_deposit)
            }
            (PnlFactorKind::MaxAfterDeposit, false) => {
                Ok(self.config.max_pnl_factor_for_short_deposit)
            }
            (PnlFactorKind::MaxAfterWithdrawal, true) => {
                Ok(self.config.max_pnl_factor_for_long_withdrawal)
            }
            (PnlFactorKind::MaxAfterWithdrawal, false) => {
                Ok(self.config.max_pnl_factor_for_short_withdrawal)
            }
            (PnlFactorKind::MaxForTrader, true) => Ok(self.config.max_pnl_factor_for_long_trader),
            (PnlFactorKind::MaxForTrader, false) => Ok(self.config.max_pnl_factor_for_short_trader),
            (PnlFactorKind::ForAdl, true) => Ok(self.config.max_pnl_factor_for_long_adl),
            (PnlFactorKind::ForAdl, false) => Ok(self.config.max_pnl_factor_for_short_adl),
            _ => Err(gmsol_model::Error::invalid_argument("missing pool kind")),
        }
    }

    fn reserve_factor(&self) -> gmsol_model::Result<Self::Num> {
        Ok(self.config.reserve_factor)
    }
}
