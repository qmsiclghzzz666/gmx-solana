use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::{
        initialize_mint2, spl_token_2022::extension::ExtensionType, InitializeMint2, Token2022,
    },
    token_interface::{
        find_mint_account_size, non_transferable_mint_initialize, NonTransferableMintInitialize,
    },
};

use crate::{constants, states::Store};

/// The accounts defintions for the `initialize_gt` instruction.
#[derive(Accounts)]
pub struct InitializeGT<'info> {
    /// Payer.
    #[account(mut)]
    pub payer: Signer<'info>,
    /// Store.
    pub store: AccountLoader<'info, Store>,
    /// GT Mint.
    /// CHECK: should be checked and initialized by the token program.
    #[account(
        init,
        space = find_mint_account_size(Some(&vec![
            ExtensionType::NonTransferable,
        ]))?,
        payer = payer,
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

pub(crate) fn initialize_gt(ctx: Context<InitializeGT>) -> Result<()> {
    ctx.accounts.enable_non_transferable_mint()?;
    ctx.accounts.initialize_mint()?;
    Ok(())
}

impl<'info> InitializeGT<'info> {
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

    fn initialize_mint(&self) -> Result<()> {
        let ctx = CpiContext::new(
            self.token_program.to_account_info(),
            InitializeMint2 {
                mint: self.gt_mint.to_account_info(),
            },
        );
        initialize_mint2(
            ctx.with_signer(&[&self.store.load()?.pda_seeds()]),
            0,
            &self.store.key(),
            None,
        )
    }
}
