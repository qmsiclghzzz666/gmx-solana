use num_traits::{CheckedAdd, Zero};

pub(super) struct Debt<T> {
    pool: T,
    claimable_collateral: T,
}

impl<T: Zero> Default for Debt<T> {
    fn default() -> Self {
        Self {
            pool: Zero::zero(),
            claimable_collateral: Zero::zero(),
        }
    }
}

impl<T> Debt<T>
where
    T: CheckedAdd + Zero,
{
    pub(super) fn add_pool_debt(&mut self, debt: &T) -> crate::Result<()> {
        self.pool = self
            .pool
            .checked_add(debt)
            .ok_or(crate::Error::Computation("adding pool debt"))?;
        Ok(())
    }

    pub(super) fn pay_for_pool_debt<U>(
        &mut self,
        f: impl FnOnce(&mut T) -> crate::Result<U>,
        is_insolvent_close_allowed: bool,
    ) -> crate::Result<U> {
        let output = f(&mut self.pool)?;
        if self.pool.is_zero() || is_insolvent_close_allowed {
            Ok(output)
        } else {
            Err(crate::Error::InsufficientFundsToPayForCosts)
        }
    }

    // FIXME: use it.
    #[allow(unused)]
    pub(super) fn add_claimable_collateral_debt(&mut self, debt: &T) -> crate::Result<()> {
        self.claimable_collateral = self
            .claimable_collateral
            .checked_add(debt)
            .ok_or(crate::Error::Computation("adding pool debt"))?;
        Ok(())
    }

    pub(super) fn pay_for_claimable_collateral_debt<U>(
        &mut self,
        f: impl FnOnce(&mut T) -> crate::Result<U>,
        is_insolvent_close_allowed: bool,
    ) -> crate::Result<U> {
        let output = f(&mut self.pool)?;
        if self.claimable_collateral.is_zero() || is_insolvent_close_allowed {
            Ok(output)
        } else {
            Err(crate::Error::InsufficientFundsToPayForCosts)
        }
    }
}
