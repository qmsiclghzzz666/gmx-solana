use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::Token};

use gmsol_store::{
    constants::MARKET_DECIMALS,
    cpi::{
        accounts::{ClaimFeesFromMarket, MarketTransferOut, PrepareAssociatedTokenAccount},
        claim_fees_from_market, market_transfer_out, prepare_associated_token_account,
    },
    program::GmsolStore,
    states::Store,
};

use crate::{states::Controller, utils::ControllerSeeds, ExchangeError};

/// The accounts definition for [`toggle_feature`](crate::gmsol_exchange::toggle_feature).
#[derive(Accounts)]
pub struct ClaimFees<'info> {
    /// Authority.
    #[account(mut)]
    pub authority: Signer<'info>,
    /// Store.
    pub store: AccountLoader<'info, Store>,
    /// Controller.
    #[account(
        mut,
        has_one = store,
        seeds = [
            crate::constants::CONTROLLER_SEED,
            store.key().as_ref(),
        ],
        bump = controller.load()?.bump,
    )]
    pub controller: AccountLoader<'info, Controller>,
    /// The token to claim.
    /// CHECK: check by CPI.
    pub token: UncheckedAccount<'info>,
    /// Market to claim from.
    /// CHECK: check by CPI.
    #[account(mut)]
    pub market: UncheckedAccount<'info>,
    /// Vault.
    /// CHECK: check by CPI.
    #[account(mut)]
    pub vault: UncheckedAccount<'info>,
    /// The receiver account.
    /// CHECK: only use as an identifier.
    pub receiver: UncheckedAccount<'info>,
    /// The treasury account.
    /// CHECK: only use as an identifier.
    pub treasury: UncheckedAccount<'info>,
    /// Receiver token account.
    /// CHECK: check by CPI.
    #[account(mut)]
    pub receiver_token_account: UncheckedAccount<'info>,
    /// Treasury token account.
    /// CHECK: check by CPI.
    #[account(mut)]
    pub treasury_token_account: UncheckedAccount<'info>,
    /// The store program.
    pub store_program: Program<'info, GmsolStore>,
    /// The [`System`] program.
    pub system_program: Program<'info, System>,
    /// The [`Token`] program.
    pub token_program: Program<'info, Token>,
    /// The [`AssociatedToken`] program.
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub(crate) fn claim_fees(ctx: Context<ClaimFees>) -> Result<()> {
    ctx.accounts.validate_the_authority()?;

    let store = ctx.accounts.store.key();
    let controller = ControllerSeeds::find(&store);

    // Check and prepare the token accounts.
    ctx.accounts.prepare_token_account(true)?;
    ctx.accounts.prepare_token_account(false)?;

    let treasury_factor = ctx.accounts.store.load()?.treasury_factor();

    // Claim fees.
    let amount = ctx.accounts.claim_fees(&controller)?;

    let mut amount_for_treasury: u64 = gmsol_model::utils::apply_factor::<_, { MARKET_DECIMALS }>(
        &u128::from(amount),
        &treasury_factor,
    )
    .ok_or(error!(ExchangeError::InvalidArgument))?
    .try_into()
    .map_err(|_| ExchangeError::AmountOverflow)?;
    amount_for_treasury = amount_for_treasury.max(amount);

    let amount_for_receiver = amount.saturating_sub(amount_for_treasury);

    // Transfer out.
    ctx.accounts
        .market_transfer_out(true, amount_for_receiver, &controller)?;
    ctx.accounts
        .market_transfer_out(false, amount_for_treasury, &controller)?;
    Ok(())
}

impl<'info> ClaimFees<'info> {
    fn validate_the_authority(&self) -> Result<()> {
        self.store
            .load()?
            .validate_claim_fees_address(self.authority.key)
    }

    fn prepare_token_account(&self, is_receiver: bool) -> Result<()> {
        let store = self.store.load()?;
        let account = if is_receiver {
            self.receiver_token_account.to_account_info()
        } else {
            self.treasury_token_account.to_account_info()
        };
        let owner = if is_receiver {
            let expected = store.receiver();
            require_eq!(
                expected,
                self.receiver.key(),
                ExchangeError::InvalidArgument
            );
            self.receiver.to_account_info()
        } else {
            let expected = store.treasury();
            require_eq!(
                expected,
                self.treasury.key(),
                ExchangeError::InvalidArgument
            );
            self.treasury.to_account_info()
        };
        let ctx = CpiContext::new(
            self.store_program.to_account_info(),
            PrepareAssociatedTokenAccount {
                payer: self.authority.to_account_info(),
                owner,
                mint: self.token.to_account_info(),
                account,
                system_program: self.system_program.to_account_info(),
                token_program: self.token_program.to_account_info(),
                associated_token_program: self.associated_token_program.to_account_info(),
            },
        );
        prepare_associated_token_account(ctx)
    }

    fn claim_fees(&self, controller: &ControllerSeeds) -> Result<u64> {
        let ctx = CpiContext::new(
            self.store_program.to_account_info(),
            ClaimFeesFromMarket {
                authority: self.controller.to_account_info(),
                store: self.store.to_account_info(),
                market: self.market.to_account_info(),
            },
        );
        let token = self.token.key();
        let amount =
            claim_fees_from_market(ctx.with_signer(&[&controller.as_seeds()]), token)?.get();
        Ok(amount)
    }

    fn market_transfer_out(
        &self,
        to_receiver: bool,
        amount: u64,
        controller: &ControllerSeeds,
    ) -> Result<()> {
        if amount == 0 {
            return Ok(());
        }

        let to = if to_receiver {
            self.receiver_token_account.to_account_info()
        } else {
            self.treasury_token_account.to_account_info()
        };

        let ctx = CpiContext::new(
            self.store_program.to_account_info(),
            MarketTransferOut {
                authority: self.controller.to_account_info(),
                store: self.store.to_account_info(),
                market: self.market.to_account_info(),
                to,
                vault: self.vault.to_account_info(),
                token_program: self.token_program.to_account_info(),
            },
        );

        market_transfer_out(ctx.with_signer(&[&controller.as_seeds()]), amount)
    }
}
