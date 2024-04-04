use anchor_lang::prelude::*;
use anchor_spl::token::{Burn, MintTo};

use crate::states::DataStore;

pub(crate) struct TransferUtils<'a, 'info> {
    store: &'a Account<'info, DataStore>,
    token_program: AccountInfo<'info>,
    mint: AccountInfo<'info>,
}

impl<'a, 'info> TransferUtils<'a, 'info> {
    pub(crate) fn new(
        token_program: AccountInfo<'info>,
        store: &'a Account<'info, DataStore>,
        mint: AccountInfo<'info>,
    ) -> Self {
        Self {
            token_program,
            mint,
            store,
        }
    }

    pub(crate) fn mint_to(&self, to: &AccountInfo<'info>, amount: u64) -> Result<()> {
        anchor_spl::token::mint_to(
            self.mint_to_ctx(to).with_signer(&[&self.store.pda_seeds()]),
            amount,
        )
    }

    pub(crate) fn burn_from(&self, from: &AccountInfo<'info>, amount: u64) -> Result<()> {
        anchor_spl::token::burn(
            self.burn_ctx(from).with_signer(&[&self.store.pda_seeds()]),
            amount,
        )
    }

    fn mint_to_ctx(
        &self,
        receiver: &AccountInfo<'info>,
    ) -> CpiContext<'_, '_, '_, 'info, MintTo<'info>> {
        CpiContext::new(
            self.token_program.clone(),
            MintTo {
                mint: self.mint.to_account_info(),
                to: receiver.clone(),
                authority: self.store.to_account_info(),
            },
        )
    }

    fn burn_ctx(&self, vault: &AccountInfo<'info>) -> CpiContext<'_, '_, '_, 'info, Burn<'info>> {
        CpiContext::new(
            self.token_program.clone(),
            Burn {
                mint: self.mint.to_account_info(),
                from: vault.clone(),
                authority: self.store.to_account_info(),
            },
        )
    }
}
