use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use typed_builder::TypedBuilder;

use crate::{
    states::{Market, Store},
    CoreError,
};

/// Operation for transferring funds into market valut.
#[derive(TypedBuilder)]
pub(crate) struct MarketTransferInOperation<'a, 'info> {
    store: &'a AccountLoader<'info, Store>,
    market: &'a AccountLoader<'info, Market>,
    from: AccountInfo<'info>,
    from_authority: AccountInfo<'info>,
    vault: &'a Account<'info, TokenAccount>,
    amount: u64,
    token_program: AccountInfo<'info>,
    signer_seeds: &'a [&'a [u8]],
}

impl<'a, 'info> MarketTransferInOperation<'a, 'info> {
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

/// Operation for transferring funds out of market vault.
#[derive(TypedBuilder)]
pub(crate) struct MarketTransferOutOperation<'a, 'info> {
    store: &'a AccountLoader<'info, Store>,
    market: &'a AccountLoader<'info, Market>,
    amount: u64,
    decimals: u8,
    to: AccountInfo<'info>,
    token_mint: AccountInfo<'info>,
    vault: AccountInfo<'info>,
    token_program: AccountInfo<'info>,
}

impl<'a, 'info> MarketTransferOutOperation<'a, 'info> {
    pub(crate) fn execute(self) -> Result<()> {
        use crate::utils::internal::TransferUtils;

        {
            let market = self.market.load()?;
            let meta = market.validated_meta(&self.store.key())?;
            require!(
                meta.is_collateral_token(&self.token_mint.key()),
                CoreError::InvalidCollateralToken
            );
        }

        let amount = self.amount;
        if amount != 0 {
            let decimals = self.decimals;
            TransferUtils::new(
                self.token_program.to_account_info(),
                self.store,
                self.token_mint.to_account_info(),
            )
            .transfer_out(self.vault.to_account_info(), self.to, amount, decimals)?;
            let token = &self.token_mint.key();
            self.market
                .load_mut()?
                .record_transferred_out_by_token(token, amount)?;
        }

        Ok(())
    }
}
