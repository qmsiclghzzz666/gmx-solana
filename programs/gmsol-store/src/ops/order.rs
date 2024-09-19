use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use typed_builder::TypedBuilder;

use crate::{
    states::{
        order::{OrderKind, OrderParamsV2, OrderV2, TokenAccounts},
        position::PositionKind,
        HasMarketMeta, Market, NonceBytes, Position, Store,
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
    /// Size delta value.
    pub size_delta_value: u128,
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

    pub(crate) fn increase(
        self,
    ) -> CreateIncreaseOrderOpBuilder<'a, 'info, ((CreateOrderOps<'a, 'info>,), (), (), (), (), ())>
    {
        CreateIncreaseOrderOp::builder().common(self)
    }

    pub(crate) fn decrease(
        self,
    ) -> CreateDecreaseOrderOpBuilder<'a, 'info, ((CreateOrderOps<'a, 'info>,), (), (), (), (), ())>
    {
        CreateDecreaseOrderOp::builder().common(self)
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
                market_token,
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

            *market_token = self.market.load()?.meta().market_token_mint;

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

/// Create Increase Order.
#[derive(TypedBuilder)]
pub(crate) struct CreateIncreaseOrderOp<'a, 'info> {
    common: CreateOrderOps<'a, 'info>,
    position: AccountLoader<'info, Position>,
    position_bump: u8,
    initial_collateral_token: &'a Account<'info, TokenAccount>,
    long_token: &'a Account<'info, TokenAccount>,
    short_token: &'a Account<'info, TokenAccount>,
}

impl<'a, 'info> CreateIncreaseOrderOp<'a, 'info> {
    pub(crate) fn execute(self) -> Result<()> {
        self.common.validate()?;
        self.validate_params_excluding_swap()?;

        let collateral_token = if self.common.params.is_collateral_long {
            self.common.market.load()?.meta().long_token_mint
        } else {
            self.common.market.load()?.meta().short_token_mint
        };

        self.common.init_with(|create, tokens, params| {
            tokens
                .initial_collateral
                .init(self.initial_collateral_token);
            tokens.long_token.init(self.long_token);
            tokens.short_token.init(self.short_token);
            params.init_increase(
                create.is_long,
                create.kind,
                collateral_token,
                create.initial_collateral_delta_amount,
                create.trigger_price,
                create.acceptable_price,
            )?;
            Ok((self.initial_collateral_token.mint, collateral_token))
        })?;

        let store = self.common.store.key();
        let market_token = self.common.market.load()?.meta().market_token_mint;
        validate_and_initialize_position_if_needed(
            &self.position,
            self.position_bump,
            self.common.params.to_position_kind()?,
            self.common.owner.key,
            &collateral_token,
            &market_token,
            &store,
        )?;
        Ok(())
    }

    fn validate_params_excluding_swap(&self) -> Result<()> {
        require!(
            self.common.params.kind.is_increase_position(),
            CoreError::Internal
        );
        require!(
            self.common.params.initial_collateral_delta_amount != 0,
            CoreError::EmptyOrder
        );
        require!(
            self.common.params.size_delta_value != 0,
            CoreError::EmptyOrder
        );
        require_gte!(
            self.initial_collateral_token.amount,
            self.common.params.initial_collateral_delta_amount,
            CoreError::NotEnoughTokenAmount
        );
        require_eq!(
            self.common.market.load()?.meta().long_token_mint,
            self.long_token.mint,
            CoreError::TokenMintMismatched
        );
        require_eq!(
            self.common.market.load()?.meta().short_token_mint,
            self.short_token.mint,
            CoreError::TokenMintMismatched
        );
        Ok(())
    }
}

fn validate_and_initialize_position_if_needed(
    position: &AccountLoader<'_, Position>,
    bump: u8,
    kind: PositionKind,
    owner: &Pubkey,
    collateral_token: &Pubkey,
    market_token: &Pubkey,
    store: &Pubkey,
) -> Result<()> {
    match position.load_init() {
        Ok(mut position) => {
            position.try_init(kind, bump, *store, owner, market_token, collateral_token)?;
        }
        Err(Error::AnchorError(err)) => {
            if err.error_code_number != ErrorCode::AccountDiscriminatorAlreadySet as u32 {
                return Err(Error::AnchorError(err));
            }
        }
        Err(err) => {
            return Err(err);
        }
    }
    validate_position(
        &*position.load()?,
        bump,
        kind,
        owner,
        collateral_token,
        market_token,
        store,
    )?;
    Ok(())
}

fn validate_position(
    position: &Position,
    bump: u8,
    kind: PositionKind,
    owner: &Pubkey,
    collateral_token: &Pubkey,
    market_token: &Pubkey,
    store: &Pubkey,
) -> Result<()> {
    require_eq!(position.bump, bump, CoreError::InvalidPosition);
    require_eq!(position.kind()?, kind, CoreError::InvalidPosition);
    require_eq!(position.owner, *owner, CoreError::InvalidPosition);
    require_eq!(
        position.collateral_token,
        *collateral_token,
        CoreError::InvalidPosition
    );
    require_eq!(
        position.market_token,
        *market_token,
        CoreError::InvalidPosition
    );
    require_eq!(position.store, *store, CoreError::InvalidPosition);
    Ok(())
}

/// Create Decrease Order.
#[derive(TypedBuilder)]
pub(crate) struct CreateDecreaseOrderOp<'a, 'info> {
    common: CreateOrderOps<'a, 'info>,
    position: AccountLoader<'info, Position>,
    position_bump: u8,
    final_output_token: &'a Account<'info, TokenAccount>,
    long_token: &'a Account<'info, TokenAccount>,
    short_token: &'a Account<'info, TokenAccount>,
}

impl<'a, 'info> CreateDecreaseOrderOp<'a, 'info> {
    pub(crate) fn execute(self) -> Result<()> {
        self.common.validate()?;
        self.validate_params_excluding_swap()?;

        let collateral_token = if self.common.params.is_collateral_long {
            self.common.market.load()?.meta().long_token_mint
        } else {
            self.common.market.load()?.meta().short_token_mint
        };

        self.common.init_with(|create, tokens, params| {
            tokens.final_output_token.init(self.final_output_token);
            tokens.long_token.init(self.long_token);
            tokens.short_token.init(self.short_token);
            params.init_decrease(
                create.is_long,
                create.kind,
                collateral_token,
                create.initial_collateral_delta_amount,
                create.trigger_price,
                create.acceptable_price,
            )?;
            Ok((collateral_token, self.final_output_token.mint))
        })?;

        let store = self.common.store.key();
        let market_token = self.common.market.load()?.meta().market_token_mint;
        validate_position(
            &*self.position.load()?,
            self.position_bump,
            self.common.params.to_position_kind()?,
            self.common.owner.key,
            &collateral_token,
            &market_token,
            &store,
        )?;
        Ok(())
    }

    fn validate_params_excluding_swap(&self) -> Result<()> {
        require!(
            self.common.params.kind.is_decrease_position(),
            CoreError::Internal
        );
        require_eq!(
            self.common.market.load()?.meta().long_token_mint,
            self.long_token.mint,
            CoreError::TokenMintMismatched
        );
        require_eq!(
            self.common.market.load()?.meta().short_token_mint,
            self.short_token.mint,
            CoreError::TokenMintMismatched
        );
        Ok(())
    }
}
