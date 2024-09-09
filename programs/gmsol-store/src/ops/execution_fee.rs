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
pub(crate) struct PayExecutionFeeOps<'a, 'info> {
    payer: AccountInfo<'info>,
    receiver: AccountInfo<'info>,
    execution_lamports: u64,
    system_program: AccountInfo<'info>,
    signer_seeds: &'a [&'a [u8]],
}

impl<'a, 'info> PayExecutionFeeOps<'a, 'info> {
    pub(crate) fn execute(self) -> Result<()> {
        use anchor_lang::system_program::{transfer, Transfer};

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

        if self.execution_lamports != 0 {
            transfer(
                CpiContext::new(
                    self.system_program,
                    Transfer {
                        from: self.payer,
                        to: self.receiver,
                    },
                )
                .with_signer(&[self.signer_seeds]),
                self.execution_lamports,
            )?;
        }

        Ok(())
    }
}
