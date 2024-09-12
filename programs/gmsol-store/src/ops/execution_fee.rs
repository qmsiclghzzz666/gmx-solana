use anchor_lang::prelude::*;
use typed_builder::TypedBuilder;

use crate::CoreError;

/// Transfer execution fee.
#[derive(TypedBuilder)]
pub(crate) struct TransferExecutionFeeOps<'info> {
    payment: AccountInfo<'info>,
    payer: AccountInfo<'info>,
    execution_lamports: u64,
    system_program: AccountInfo<'info>,
}

impl<'info> TransferExecutionFeeOps<'info> {
    pub(crate) fn execute(self) -> Result<()> {
        use anchor_lang::system_program::{transfer, Transfer};

        if self.execution_lamports != 0 {
            transfer(
                CpiContext::new(
                    self.system_program,
                    Transfer {
                        from: self.payer,
                        to: self.payment,
                    },
                ),
                self.execution_lamports,
            )?;
        }

        Ok(())
    }
}

/// Pay execution fee.
#[derive(TypedBuilder)]
pub(crate) struct PayExecutionFeeOps<'info> {
    payer: AccountInfo<'info>,
    receiver: AccountInfo<'info>,
    execution_lamports: u64,
}

impl<'info> PayExecutionFeeOps<'info> {
    pub(crate) fn execute(self) -> Result<()> {
        let rent = Rent::get()?;
        let remaining_lamports = self
            .payer
            .lamports()
            .saturating_sub(self.execution_lamports);
        require_gte!(
            remaining_lamports,
            rent.minimum_balance(self.payer.data_len()),
            CoreError::NotEnoughExecutionFee,
        );

        let amount = self.execution_lamports;
        if amount != 0 {
            msg!("paying execution fee: {}", amount);
            self.payer.sub_lamports(amount)?;
            self.receiver.add_lamports(amount)?;
        }

        Ok(())
    }
}
