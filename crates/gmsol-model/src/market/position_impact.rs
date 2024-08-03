use crate::{
    action::distribute_position_impact::DistributePositionImpact,
    params::{position::PositionImpactDistributionParams, PriceImpactParams},
    Balance, BaseMarket, BaseMarketMut, Pool,
};

/// A market with position impact pool.
pub trait PositionImpactMarket<const DECIMALS: u8>: BaseMarket<DECIMALS> {
    /// Get position impact pool.
    fn position_impact_pool(&self) -> crate::Result<&Self::Pool>;

    /// Get the position impact params.
    fn position_impact_params(&self) -> crate::Result<PriceImpactParams<Self::Num>>;

    /// Get position impact distribution params.
    fn position_impact_distribution_params(
        &self,
    ) -> crate::Result<PositionImpactDistributionParams<Self::Num>>;
}

/// A mutable market with position impact pool.
pub trait PositionImpactMarketMut<const DECIMALS: u8>:
    BaseMarketMut<DECIMALS> + PositionImpactMarket<DECIMALS>
{
    /// Get position impact pool mutably.
    fn position_impact_pool_mut(&mut self) -> crate::Result<&mut Self::Pool>;

    /// Get the just passed time in seconds for the given kind of clock.
    fn just_passed_in_seconds_for_position_impact_distribution(&mut self) -> crate::Result<u64>;
}

impl<'a, M: PositionImpactMarket<DECIMALS>, const DECIMALS: u8> PositionImpactMarket<DECIMALS>
    for &'a mut M
{
    fn position_impact_pool(&self) -> crate::Result<&Self::Pool> {
        (**self).position_impact_pool()
    }

    fn position_impact_params(&self) -> crate::Result<PriceImpactParams<Self::Num>> {
        (**self).position_impact_params()
    }

    fn position_impact_distribution_params(
        &self,
    ) -> crate::Result<PositionImpactDistributionParams<Self::Num>> {
        (**self).position_impact_distribution_params()
    }
}

impl<'a, M: PositionImpactMarketMut<DECIMALS>, const DECIMALS: u8> PositionImpactMarketMut<DECIMALS>
    for &'a mut M
{
    fn position_impact_pool_mut(&mut self) -> crate::Result<&mut Self::Pool> {
        (**self).position_impact_pool_mut()
    }

    fn just_passed_in_seconds_for_position_impact_distribution(&mut self) -> crate::Result<u64> {
        (**self).just_passed_in_seconds_for_position_impact_distribution()
    }
}

/// Extension trait of [`PositionImpactMarket`].
pub trait PositionImpactMarketExt<const DECIMALS: u8>: PositionImpactMarket<DECIMALS> {
    /// Get position impact pool amount.
    #[inline]
    fn position_impact_pool_amount(&self) -> crate::Result<Self::Num> {
        self.position_impact_pool()?.long_amount()
    }
}

impl<M: PositionImpactMarket<DECIMALS> + ?Sized, const DECIMALS: u8>
    PositionImpactMarketExt<DECIMALS> for M
{
}

/// Extension trait of [`PositionImpactMarketMut`].
pub trait PositionImpactMarketMutExt<const DECIMALS: u8>:
    PositionImpactMarketMut<DECIMALS>
{
    /// Apply delta to the position impact pool.
    fn apply_delta_to_position_impact_pool(&mut self, delta: &Self::Signed) -> crate::Result<()> {
        self.position_impact_pool_mut()?
            .apply_delta_to_long_amount(delta)
    }

    /// Get position impact pool amount.
    #[inline]
    fn position_impact_pool_amount(&self) -> crate::Result<Self::Num> {
        self.position_impact_pool()?.long_amount()
    }

    /// Create a [`DistributePositionImpact`] action.
    fn distribute_position_impact(
        &mut self,
    ) -> crate::Result<DistributePositionImpact<&mut Self, DECIMALS>>
    where
        Self: Sized,
    {
        Ok(DistributePositionImpact::from(self))
    }
}

impl<M: PositionImpactMarketMut<DECIMALS> + ?Sized, const DECIMALS: u8>
    PositionImpactMarketMutExt<DECIMALS> for M
{
}
