use anchor_lang::prelude::*;
use borsh::BorshSerialize;

use gmsol_model::action::{
    distribute_position_impact::DistributePositionImpactReport,
    update_borrowing_state::UpdateBorrowingReport, update_funding_state::UpdateFundingReport,
};

use super::Event;

/// Market fees updated event.
#[event]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct MarketFeesUpdated {
    /// Position impact distribution report.
    pub position_impact_distribution: DistributePositionImpactReport<u128>,
    /// Update borrowing state report.
    pub update_borrowing_state: UpdateBorrowingReport<u128>,
    /// Update funding state report.
    pub update_funding_state: UpdateFundingReport<u128, i128>,
}

impl gmsol_utils::InitSpace for MarketFeesUpdated {
    const INIT_SPACE: usize = DistributePositionImpactReport::<u128>::INIT_SPACE
        + UpdateBorrowingReport::<u128>::INIT_SPACE
        + UpdateFundingReport::<u128, i128>::INIT_SPACE;
}

impl Event for MarketFeesUpdated {}

impl MarketFeesUpdated {
    /// Create from reports.
    pub fn from_reports(
        position_impact_distribution: DistributePositionImpactReport<u128>,
        update_borrowing_state: UpdateBorrowingReport<u128>,
        update_funding_state: UpdateFundingReport<u128, i128>,
    ) -> Self {
        Self {
            position_impact_distribution,
            update_borrowing_state,
            update_funding_state,
        }
    }
}
