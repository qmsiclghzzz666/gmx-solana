use anchor_lang::prelude::*;
use typed_builder::TypedBuilder;

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

        let lamports = self.payment.get_lamports();
        let amount = self.execution_lamports.saturating_sub(lamports);
        if amount != 0 {
            transfer(
                CpiContext::new(
                    self.system_program,
                    Transfer {
                        from: self.payer,
                        to: self.payment,
                    },
                ),
                amount,
            )?;
        }

        Ok(())
    }
}
