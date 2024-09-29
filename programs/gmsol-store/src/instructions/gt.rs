use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::{
        initialize_mint2, spl_token_2022::extension::ExtensionType, InitializeMint2, Token2022,
    },
    token_interface::{
        find_mint_account_size, non_transferable_mint_initialize, NonTransferableMintInitialize,
    },
};

use crate::{constants, states::Store, utils::internal};

/// The accounts defintions for the `initialize_gt` instruction.
#[derive(Accounts)]
pub struct InitializeGT<'info> {
    /// Authority
    #[account(mut)]
    pub authority: Signer<'info>,
    /// Store.
    #[account(mut)]
    pub store: AccountLoader<'info, Store>,
    /// GT Mint.
    /// CHECK: should be checked and initialized by the token program.
    #[account(
        init,
        space = find_mint_account_size(Some(&vec![
            ExtensionType::NonTransferable,
        ]))?,
        payer = authority,
        seeds = [
            constants::GT_MINT_SEED,
            store.key().as_ref(),
        ],
        bump,
        owner = token_program.key(),
    )]
    pub gt_mint: UncheckedAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token2022>,
}

pub(crate) fn unchecked_initialize_gt(
    ctx: Context<InitializeGT>,
    decimals: u8,
    mint_base_value: u128,
    initial_mint_rate_factor: u128,
    decay_factor: u128,
    decay_step: u64,
) -> Result<()> {
    ctx.accounts.initialize_gt_state(
        mint_base_value,
        initial_mint_rate_factor,
        decay_factor,
        decay_step,
    )?;
    ctx.accounts.enable_non_transferable_mint()?;
    ctx.accounts.initialize_mint(decimals)?;
    Ok(())
}

impl<'info> internal::Authentication<'info> for InitializeGT<'info> {
    fn authority(&self) -> &Signer<'info> {
        &self.authority
    }

    fn store(&self) -> &AccountLoader<'info, Store> {
        &self.store
    }
}

impl<'info> InitializeGT<'info> {
    fn initialize_gt_state(
        &self,
        mint_base_value: u128,
        initial_mint_rate_factor: u128,
        decay_factor: u128,
        decay_step: u64,
    ) -> Result<()> {
        let mut store = self.store.load_mut()?;
        store.gt_mut().init(
            mint_base_value,
            initial_mint_rate_factor,
            decay_factor,
            decay_step,
        )?;
        Ok(())
    }
    fn enable_non_transferable_mint(&self) -> Result<()> {
        let ctx = CpiContext::new(
            self.token_program.to_account_info(),
            NonTransferableMintInitialize {
                token_program_id: self.token_program.to_account_info(),
                mint: self.gt_mint.to_account_info(),
            },
        );
        non_transferable_mint_initialize(ctx.with_signer(&[&self.store.load()?.pda_seeds()]))
    }

    fn initialize_mint(&self, decimals: u8) -> Result<()> {
        let ctx = CpiContext::new(
            self.token_program.to_account_info(),
            InitializeMint2 {
                mint: self.gt_mint.to_account_info(),
            },
        );
        initialize_mint2(
            ctx.with_signer(&[&self.store.load()?.pda_seeds()]),
            decimals,
            &self.store.key(),
            None,
        )
    }
}
