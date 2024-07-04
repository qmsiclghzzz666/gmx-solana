use anchor_lang::{solana_program::account_info::AccountInfo, system_program};

/// Return whether the account must be uninitialized.
pub fn must_be_uninitialized<'info>(account: &impl AsRef<AccountInfo<'info>>) -> bool {
    let info = account.as_ref();
    *info.owner == system_program::ID
}
