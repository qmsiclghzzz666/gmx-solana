use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use typed_builder::TypedBuilder;

use crate::{
    states::{
        order::{OrderKind, OrderParamsV2, OrderV2, TokenAccounts},
        position::PositionKind,
        HasMarketMeta, Market, NonceBytes, Store,
    },
    CoreError,
};

/// Create Order Params.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateOrderParams {
    /// Order Kind.
    pub kind: OrderKind,
    /// Execution fee in lamports.
    pub execution_fee: u64,
    /// The length of the swap path.
    pub swap_path_length: u8,
    /// Initial collateral / swap in token amount.
    pub initial_collateral_delta_amount: u64,
    /// Is long.
    pub is_long: bool,
    /// Is collateral or the swap out token the long token.
    pub is_collateral_long: bool,
    /// Min output amount or value.
    pub min_output: Option<u128>,
    /// Trigger price.
    pub trigger_price: Option<u128>,
    /// Acceptable price.
    pub acceptable_price: Option<u128>,
}

impl CreateOrderParams {
    /// Get the related position kind.
    pub fn to_position_kind(&self) -> Result<PositionKind> {
        if self.kind.is_swap() {
            return err!(CoreError::PositionItNotRequired);
        }
        if self.is_long {
            Ok(PositionKind::Long)
        } else {
            Ok(PositionKind::Short)
        }
    }

    /// Get the collateral token or swap out token address.
    pub fn collateral_token<'a>(&'a self, meta: &'a impl HasMarketMeta) -> &'a Pubkey {
        let meta = meta.market_meta();
        if self.is_long {
            &meta.long_token_mint
        } else {
            &meta.short_token_mint
        }
    }
}

/// Create Order Ops
#[derive(TypedBuilder)]
pub(crate) struct CreateOrderOps<'a, 'info> {
    order: AccountLoader<'info, OrderV2>,
    market: AccountLoader<'info, Market>,
    store: AccountLoader<'info, Store>,
    owner: AccountInfo<'info>,
    nonce: &'a NonceBytes,
    bump: u8,
    params: &'a CreateOrderParams,
    swap_path: &'info [AccountInfo<'info>],
}

impl<'a, 'info> CreateOrderOps<'a, 'info> {
    pub(crate) fn swap(
        self,
    ) -> CreateSwapOrderOpBuilder<'a, 'info, ((CreateOrderOps<'a, 'info>,), (), ())> {
        CreateSwapOrderOp::builder().common(self)
    }

    fn validate(&self) -> Result<()> {
        self.market.load()?.validate(&self.store.key())?;
        require_gte!(
            self.params.execution_fee,
            OrderV2::MIN_EXECUTION_LAMPORTS,
            CoreError::NotEnoughExecutionFee
        );
        let balance = self
            .order
            .get_lamports()
            .saturating_sub(self.params.execution_fee);
        let rent = Rent::get()?;
        require!(
            rent.is_exempt(balance, OrderV2::INIT_SPACE),
            CoreError::NotEnoughExecutionFee
        );
        Ok(())
    }

    fn init_with(
        &self,
        f: impl FnOnce(
            &CreateOrderParams,
            &mut TokenAccounts,
            &mut OrderParamsV2,
        ) -> Result<(Pubkey, Pubkey)>,
    ) -> Result<()> {
        let id = self.market.load_mut()?.state_mut().next_order_id()?;
        {
            let mut order = self.order.load_init()?;
            let OrderV2 {
                header,
                tokens,
                params,
                swap,
                updated_at,
                updated_at_slot,
                ..
            } = &mut *order;

            header.init(
                id,
                self.store.key(),
                self.market.key(),
                self.owner.key(),
                *self.nonce,
                self.bump,
            );

            let clock = Clock::get()?;
            *updated_at = clock.unix_timestamp;
            *updated_at_slot = clock.slot;

            let (from, to) = (f)(self.params, tokens, params)?;

            let market = self.market.load()?;
            let meta = market.meta();
            let swap_path = self.swap_path;
            swap.validate_and_init(
                meta,
                self.params.swap_path_length,
                0,
                swap_path,
                &self.store.key(),
                (&from, &from),
                (&to, &to),
            )?;
        }
        Ok(())
    }
}

/// Create Swap Order.
#[derive(TypedBuilder)]
pub(crate) struct CreateSwapOrderOp<'a, 'info> {
    common: CreateOrderOps<'a, 'info>,
    swap_in_token: &'a Account<'info, TokenAccount>,
    swap_out_token: &'a Account<'info, TokenAccount>,
}

impl<'a, 'info> CreateSwapOrderOp<'a, 'info> {
    pub(crate) fn execute(self) -> Result<()> {
        self.common.validate()?;
        self.validate_params_excluding_swap()?;

        self.common.init_with(|create, tokens, params| {
            tokens.initial_collateral.init(self.swap_in_token);
            tokens.final_output_token.init(self.swap_out_token);
            params.init_swap(
                create.kind,
                self.swap_out_token.mint,
                create.initial_collateral_delta_amount,
                create.min_output,
            )?;
            Ok((self.swap_in_token.mint, self.swap_out_token.mint))
        })?;
        Ok(())
    }

    fn validate_params_excluding_swap(&self) -> Result<()> {
        require!(self.common.params.kind.is_swap(), CoreError::Internal);
        require!(
            self.common.params.initial_collateral_delta_amount != 0,
            CoreError::EmptyOrder
        );
        require_gte!(
            self.swap_in_token.amount,
            self.common.params.initial_collateral_delta_amount,
            CoreError::NotEnoughTokenAmount
        );
        require!(
            self.common
                .market
                .load()?
                .meta()
                .is_collateral_token(&self.swap_out_token.mint),
            CoreError::TokenMintMismatched
        );
        Ok(())
    }
}
