use anchor_lang::prelude::*;
use anchor_spl::token::{Burn, MintTo, Transfer};

use crate::{states::Store, CoreError};

pub(crate) struct TransferUtils<'a, 'info> {
    store: &'a AccountLoader<'info, Store>,
    token_program: AccountInfo<'info>,
    mint: Option<AccountInfo<'info>>,
}

impl<'a, 'info> TransferUtils<'a, 'info> {
    pub(crate) fn new(
        token_program: AccountInfo<'info>,
        store: &'a AccountLoader<'info, Store>,
        mint: Option<AccountInfo<'info>>,
    ) -> Self {
        Self {
            token_program,
            mint,
            store,
        }
    }

    pub(crate) fn mint_to(&self, to: &AccountInfo<'info>, amount: u64) -> Result<()> {
        anchor_spl::token::mint_to(
            self.mint_to_ctx(to)?
                .with_signer(&[&self.store.load()?.pda_seeds()]),
            amount,
        )
    }

    pub(crate) fn burn_from(&self, from: &AccountInfo<'info>, amount: u64) -> Result<()> {
        anchor_spl::token::burn(
            self.burn_ctx(from)?
                .with_signer(&[&self.store.load()?.pda_seeds()]),
            amount,
        )
    }

    pub(crate) fn transfer_out(
        &self,
        from: AccountInfo<'info>,
        to: AccountInfo<'info>,
        amount: u64,
    ) -> Result<()> {
        anchor_spl::token::transfer(
            self.transfer_ctx(from, to)
                .with_signer(&[&self.store.load()?.pda_seeds()]),
            amount,
        )
    }

    fn transfer_ctx(
        &self,
        from: AccountInfo<'info>,
        to: AccountInfo<'info>,
    ) -> CpiContext<'_, '_, '_, 'info, Transfer<'info>> {
        CpiContext::new(
            self.token_program.clone(),
            Transfer {
                from,
                to,
                authority: self.store.to_account_info(),
            },
        )
    }

    fn mint_to_ctx(
        &self,
        receiver: &AccountInfo<'info>,
    ) -> Result<CpiContext<'_, '_, '_, 'info, MintTo<'info>>> {
        Ok(CpiContext::new(
            self.token_program.clone(),
            MintTo {
                mint: self
                    .mint
                    .as_ref()
                    .ok_or(CoreError::TokenMintNotProvided)?
                    .to_account_info(),
                to: receiver.clone(),
                authority: self.store.to_account_info(),
            },
        ))
    }

    fn burn_ctx(
        &self,
        vault: &AccountInfo<'info>,
    ) -> Result<CpiContext<'_, '_, '_, 'info, Burn<'info>>> {
        Ok(CpiContext::new(
            self.token_program.clone(),
            Burn {
                mint: self
                    .mint
                    .as_ref()
                    .ok_or(CoreError::TokenMintNotProvided)?
                    .to_account_info(),
                from: vault.clone(),
                authority: self.store.to_account_info(),
            },
        ))
    }
}
