use anchor_spl::associated_token::{self, get_associated_token_address_with_program_id};
use gmsol_programs::anchor_lang::{InstructionData, ToAccountMetas};
use gmsol_programs::gmsol_store::client::args;
use gmsol_programs::gmsol_store::types::CreateOrderParams as StoreCreateOrderParams;
use gmsol_programs::gmsol_store::{client::accounts, types::OrderKind};
use solana_sdk::instruction::Instruction;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::system_program;
use typed_builder::TypedBuilder;

use crate::builders::{
    utils::{generate_nonce, prepare_ata},
    NonceBytes, StoreProgram,
};
use crate::utils::serde::StringPubkey;
use crate::{AtomicInstructionGroup, IntoAtomicInstructionGroup};

use super::MIN_EXECUTION_LAMPORTS_FOR_ORDER;

/// Create Order Kind.
#[cfg_attr(serde, derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum CreateOrderKind {
    /// Market Swap.
    MarketSwap,
    /// Market Increase.
    MarketIncrease,
    /// Market Decrease.
    MarketDecrease,
    /// Limit Swap.
    LimitSwap,
    /// Limit Increase.
    LimitIncrease,
    /// Limit Decrease.
    LimitDecrease,
    /// Stop-loss Decrease.
    StopLossDecrease,
}

impl From<CreateOrderKind> for OrderKind {
    fn from(kind: CreateOrderKind) -> Self {
        match kind {
            CreateOrderKind::MarketSwap => Self::MarketSwap,
            CreateOrderKind::MarketIncrease => Self::MarketIncrease,
            CreateOrderKind::MarketDecrease => Self::MarketDecrease,
            CreateOrderKind::LimitSwap => Self::LimitSwap,
            CreateOrderKind::LimitIncrease => Self::LimitIncrease,
            CreateOrderKind::LimitDecrease => Self::LimitDecrease,
            CreateOrderKind::StopLossDecrease => Self::StopLossDecrease,
        }
    }
}

/// Swap type for decreasing position.
#[cfg_attr(serde, derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, Copy)]
pub enum DecreasePositionSwapType {
    /// Do not swap.
    NoSwap,
    /// Swap PnL token to collateral token.
    PnlTokenToCollateralToken,
    /// Swap collateral token to PnL token.
    CollateralToPnlToken,
}

impl From<DecreasePositionSwapType>
    for gmsol_programs::gmsol_store::types::DecreasePositionSwapType
{
    fn from(ty: DecreasePositionSwapType) -> Self {
        match ty {
            DecreasePositionSwapType::NoSwap => Self::NoSwap,
            DecreasePositionSwapType::PnlTokenToCollateralToken => Self::PnlTokenToCollateralToken,
            DecreasePositionSwapType::CollateralToPnlToken => Self::CollateralToPnlToken,
        }
    }
}

/// Parameters for creating an order.
#[cfg_attr(serde, derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, TypedBuilder)]
pub struct CreateOrderParams {
    /// Order Kind.
    pub kind: CreateOrderKind,
    /// Whether the order is for a long or short position.
    pub is_long: bool,
    /// Delta size in USD.
    pub size: u128,
    /// The amount of pay tokens to use.
    pub pay_token_amount: u64,
    /// Minimum amount or value of output tokens.
    ///
    /// - Minimum collateral amount for increase-position orders after swap.
    /// - Minimum swap-out amount for swap orders.
    /// - Minimum output value for decrease-position orders.
    #[cfg_attr(serde, serde(default))]
    #[builder(default)]
    pub min_output: u128,
    /// Trigger price (in unit price).
    #[cfg_attr(serde, serde(default))]
    #[builder(default, setter(strip_option))]
    pub trigger_price: Option<u128>,
    /// Acceptable price (in unit price).
    #[cfg_attr(serde, serde(default))]
    #[builder(default, setter(strip_option))]
    pub acceptable_price: Option<u128>,
    /// Decrease Position Swap Type.
    #[cfg_attr(serde, serde(default))]
    #[builder(default, setter(strip_option))]
    pub decrease_position_swap_type: Option<DecreasePositionSwapType>,
    /// Timestamp from which the order becomes valid.
    #[cfg_attr(serde, serde(default))]
    #[builder(default, setter(strip_option))]
    pub valid_from_ts: Option<i64>,
}

/// Instruction builder for the `create_order` instruction.
#[cfg_attr(serde, derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, TypedBuilder)]
pub struct CreateOrder {
    /// Program.
    pub program: StoreProgram,
    /// Payer (a.k.a. owner).
    #[builder(setter(into))]
    pub payer: StringPubkey,
    /// Reciever.
    #[cfg_attr(serde, serde(default))]
    #[builder(default, setter(strip_option, into))]
    pub receiver: Option<StringPubkey>,
    /// Nonce for the order.
    #[cfg_attr(serde, serde(default))]
    #[builder(default, setter(strip_option, into))]
    pub nonce: Option<NonceBytes>,
    /// Execution fee paid to the keeper in lamports.
    #[cfg_attr(serde, serde(default = "default_execution_lamports"))]
    #[builder(default = MIN_EXECUTION_LAMPORTS_FOR_ORDER)]
    pub execution_lamports: u64,
    /// The market token of the market in which the order will be created.
    #[builder(setter(into))]
    pub market_token: StringPubkey,
    /// Whether the collateral or the swap-out token is the long token.
    pub is_collateral_or_swap_out_token_long: bool,
    /// Order Parameters.
    pub params: CreateOrderParams,
    /// Pay token.
    #[cfg_attr(serde, serde(default))]
    #[builder(default, setter(strip_option, into))]
    pub pay_token: Option<StringPubkey>,
    /// Pay token account.
    #[cfg_attr(serde, serde(default))]
    #[builder(default, setter(strip_option, into))]
    pub pay_token_account: Option<StringPubkey>,
    /// Receive token.
    #[cfg_attr(serde, serde(default))]
    #[builder(default, setter(strip_option, into))]
    pub receive_token: Option<StringPubkey>,
    /// Swap path.
    #[cfg_attr(serde, serde(default))]
    #[builder(default, setter(into))]
    pub swap_path: Vec<StringPubkey>,
    /// Whether to unwrap the native token when receiving (e.g., convert WSOL to SOL).
    #[cfg_attr(serde, serde(default))]
    #[builder(default)]
    pub unwrap_native_on_receive: bool,
}

#[cfg(serde)]
fn default_execution_lamports() -> u64 {
    MIN_EXECUTION_LAMPORTS_FOR_ORDER
}

impl CreateOrder {
    fn collateral_or_swap_out_token(&self, hint: &CreateOrderHint) -> Pubkey {
        if self.is_collateral_or_swap_out_token_long {
            hint.long_token.0
        } else {
            hint.short_token.0
        }
    }
}

/// Hint for [`CreateOrder`].
#[cfg_attr(serde, derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone, TypedBuilder)]
pub struct CreateOrderHint {
    /// Long token.
    #[builder(setter(into))]
    pub long_token: StringPubkey,
    /// Short token.
    #[builder(setter(into))]
    pub short_token: StringPubkey,
}

impl IntoAtomicInstructionGroup for CreateOrder {
    type Hint = CreateOrderHint;

    fn into_atomic_instruction_group(
        self,
        hint: &Self::Hint,
    ) -> crate::Result<AtomicInstructionGroup> {
        let mut insts = AtomicInstructionGroup::default();

        let owner = self.payer.0;
        let receiver = self.receiver.as_deref().copied().unwrap_or(owner);
        let nonce = self.nonce.unwrap_or_else(generate_nonce);
        let order = self.program.find_order_address(&owner, &nonce);
        let token_program_id = anchor_spl::token::ID;

        let collateral_or_swap_out_token = self.collateral_or_swap_out_token(hint);

        let (pay_token, receive_token, long_token, short_token, is_position_order) =
            match self.params.kind {
                CreateOrderKind::MarketSwap | CreateOrderKind::LimitSwap => (
                    Some(
                        self.pay_token
                            .as_deref()
                            .copied()
                            .unwrap_or(collateral_or_swap_out_token),
                    ),
                    Some(collateral_or_swap_out_token),
                    None,
                    None,
                    false,
                ),
                CreateOrderKind::MarketIncrease | CreateOrderKind::LimitIncrease => (
                    Some(
                        self.pay_token
                            .as_deref()
                            .copied()
                            .unwrap_or(collateral_or_swap_out_token),
                    ),
                    None,
                    Some(hint.long_token.0),
                    Some(hint.short_token.0),
                    true,
                ),
                CreateOrderKind::MarketDecrease
                | CreateOrderKind::LimitDecrease
                | CreateOrderKind::StopLossDecrease => (
                    None,
                    Some(
                        self.receive_token
                            .as_deref()
                            .copied()
                            .unwrap_or(collateral_or_swap_out_token),
                    ),
                    Some(hint.long_token.0),
                    Some(hint.short_token.0),
                    true,
                ),
            };

        let pay_token_account = pay_token.as_ref().map(|token| {
            self.pay_token_account
                .as_deref()
                .copied()
                .unwrap_or_else(|| {
                    get_associated_token_address_with_program_id(&owner, token, &token_program_id)
                })
        });
        let (pay_token_escrow, prepare) =
            prepare_ata(&owner, &order, pay_token.as_ref(), &token_program_id).unzip();
        insts.extend(prepare);
        let (receive_token_escrow, prepare) =
            prepare_ata(&owner, &order, receive_token.as_ref(), &token_program_id).unzip();
        insts.extend(prepare);
        let (long_token_escrow, prepare) =
            prepare_ata(&owner, &order, long_token.as_ref(), &token_program_id).unzip();
        insts.extend(prepare);
        let (short_token_escrow, prepare) =
            prepare_ata(&owner, &order, short_token.as_ref(), &token_program_id).unzip();
        insts.extend(prepare);

        let market = self.program.find_market_address(&self.market_token);
        let user = self.program.find_user_address(&owner);
        let position = (is_position_order).then(|| {
            self.program.find_position_address(
                &owner,
                &self.market_token,
                &collateral_or_swap_out_token,
                self.params.is_long,
            )
        });
        let params = &self.params;

        let create = Instruction {
            program_id: self.program.id,
            accounts: accounts::CreateOrder {
                owner,
                receiver,
                store: self.program.store,
                market,
                user,
                order,
                position,
                initial_collateral_token: pay_token,
                final_output_token: receive_token.unwrap_or(collateral_or_swap_out_token),
                long_token,
                short_token,
                initial_collateral_token_escrow: pay_token_escrow,
                final_output_token_escrow: receive_token_escrow,
                long_token_escrow,
                short_token_escrow,
                initial_collateral_token_source: pay_token_account,
                system_program: system_program::ID,
                token_program: token_program_id,
                associated_token_program: associated_token::ID,
            }
            .to_account_metas(None),
            data: args::CreateOrder {
                nonce: nonce.to_bytes(),
                params: StoreCreateOrderParams {
                    kind: params.kind.into(),
                    decrease_position_swap_type: params.decrease_position_swap_type.map(Into::into),
                    execution_lamports: self.execution_lamports,
                    swap_path_length: self.swap_path.len() as u8,
                    initial_collateral_delta_amount: self.params.pay_token_amount,
                    size_delta_value: self.params.size,
                    is_long: self.params.is_long,
                    is_collateral_long: self.is_collateral_or_swap_out_token_long,
                    min_output: Some(self.params.min_output),
                    trigger_price: self.params.trigger_price,
                    acceptable_price: self.params.acceptable_price,
                    should_unwrap_native_token: self.unwrap_native_on_receive,
                    valid_from_ts: self.params.valid_from_ts,
                },
            }
            .data(),
        };

        insts.add(create);

        Ok(insts)
    }
}
