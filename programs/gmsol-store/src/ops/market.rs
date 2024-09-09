use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use typed_builder::TypedBuilder;

use crate::states::{Market, Store};

/// Market Transfer In.
#[derive(TypedBuilder)]
pub(crate) struct MarketTransferIn<'a, 'info> {
    store: &'a AccountLoader<'info, Store>,
    market: &'a AccountLoader<'info, Market>,
    from: AccountInfo<'info>,
    from_authority: AccountInfo<'info>,
    vault: &'a Account<'info, TokenAccount>,
    amount: u64,
    token_program: AccountInfo<'info>,
    signer_seeds: &'a [&'a [u8]],
}

impl<'a, 'info> MarketTransferIn<'a, 'info> {
    pub(crate) fn execute(self) -> Result<()> {
        use anchor_spl::token;

        self.market.load()?.validate(&self.store.key())?;

        let amount = self.amount;
        if amount != 0 {
            token::transfer(
                CpiContext::new(
                    self.token_program,
                    token::Transfer {
                        from: self.from,
                        to: self.vault.to_account_info(),
                        authority: self.from_authority,
                    },
                )
                .with_signer(&[self.signer_seeds]),
                amount,
            )?;
            let token = &self.vault.mint;
            self.market
                .load_mut()?
                .record_transferred_in_by_token(token, amount)?;
        }

        Ok(())
    }
}
