use anchor_client::solana_sdk::pubkey::Pubkey;
use data_store::states::{
    position::PositionKind, Deposit, NonceBytes, Oracle, Order, Position, Seed, Store, Withdrawal,
};
use gmx_solana_utils::to_seed;

pub use data_store::states::market::find_market_address;

use crate::utils::EVENT_AUTHORITY_SEED;

/// Find PDA for `event_authority` account.
pub fn find_event_authority_address(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[EVENT_AUTHORITY_SEED], program_id)
}

/// Find PDA for [`Store`] account.
pub fn find_store_address(key: &str, store_program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[Store::SEED, &to_seed(key)], store_program_id)
}

/// Find PDA for the controller address of exchange program.
pub fn find_controller_address(store: &Pubkey, exchange_program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[exchange::constants::CONTROLLER_SEED, store.as_ref()],
        exchange_program_id,
    )
}

/// Find PDA for [`Oracle`] account.
pub fn find_oracle_address(store: &Pubkey, index: u8, store_program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[Oracle::SEED, store.as_ref(), &[index]], store_program_id)
}

/// Find PDA for the market vault.
pub fn find_market_vault_address(
    store: &Pubkey,
    token: &Pubkey,
    store_program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            data_store::constants::MARKET_VAULT_SEED,
            store.as_ref(),
            token.as_ref(),
            &[],
        ],
        store_program_id,
    )
}

/// Find PDA for Market token mint account.
pub fn find_market_token_address(
    store: &Pubkey,
    index_token: &Pubkey,
    long_token: &Pubkey,
    short_token: &Pubkey,
    store_program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            data_store::constants::MAREKT_TOKEN_MINT_SEED,
            store.as_ref(),
            index_token.as_ref(),
            long_token.as_ref(),
            short_token.as_ref(),
        ],
        store_program_id,
    )
}

/// Create PDA for deposit.
pub fn find_deposit_address(
    store: &Pubkey,
    user: &Pubkey,
    nonce: &NonceBytes,
    store_program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[Deposit::SEED, store.as_ref(), user.as_ref(), nonce],
        store_program_id,
    )
}

/// Create PDA for withdrawal.
pub fn find_withdrawal_address(
    store: &Pubkey,
    user: &Pubkey,
    nonce: &NonceBytes,
    store_program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[Withdrawal::SEED, store.as_ref(), user.as_ref(), nonce],
        store_program_id,
    )
}

/// Create PDA for order.
pub fn find_order_address(
    store: &Pubkey,
    user: &Pubkey,
    nonce: &NonceBytes,
    store_program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[Order::SEED, store.as_ref(), user.as_ref(), nonce],
        store_program_id,
    )
}

/// Create PDA for position.
pub fn find_position_address(
    store: &Pubkey,
    user: &Pubkey,
    market_token: &Pubkey,
    collateral_token: &Pubkey,
    kind: PositionKind,
    store_program_id: &Pubkey,
) -> crate::Result<(Pubkey, u8)> {
    if matches!(kind, PositionKind::Uninitialized) {
        return Err(crate::Error::invalid_argument(
            "uninitialized position kind is not allowed",
        ));
    }
    Ok(Pubkey::find_program_address(
        &[
            Position::SEED,
            store.as_ref(),
            user.as_ref(),
            market_token.as_ref(),
            collateral_token.as_ref(),
            &[kind as u8],
        ],
        store_program_id,
    ))
}

/// Find PDA for claimable account.
pub fn find_claimable_account_pda(
    store: &Pubkey,
    mint: &Pubkey,
    user: &Pubkey,
    time_key: &[u8],
    store_program_id: &Pubkey,
) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            data_store::constants::CLAIMABLE_ACCOUNT_SEED,
            store.as_ref(),
            mint.as_ref(),
            user.as_ref(),
            time_key,
        ],
        store_program_id,
    )
}
