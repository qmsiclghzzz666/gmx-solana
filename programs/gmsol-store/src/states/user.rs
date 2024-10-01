use anchor_lang::prelude::*;

use crate::CoreError;

use super::Seed;

/// Header of `User` Account.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct UserHeader {
    /// Version of the user account.
    pub(crate) version: u8,
    /// The bump seed.
    pub(crate) bump: u8,
    flags: UserFlagValue,
    padding_0: [u8; 13],
    /// The owner of this user account.
    pub(crate) owner: Pubkey,
    /// The store.
    pub(crate) store: Pubkey,
    /// Referral.
    pub(crate) referral: Referral,
    /// GT State.
    pub(crate) gt: GTState,
    reserved: [u8; 128],
}

impl UserHeader {
    /// Get flag.
    fn flag(&self, flag: UserFlag) -> bool {
        let map = UserFlagMap::from_value(self.flags);
        map.get(flag as usize)
    }

    /// Set flag.
    /// Return the previous value.
    fn set_flag(&mut self, flag: UserFlag, value: bool) -> bool {
        let mut map = UserFlagMap::from_value(self.flags);
        let previous = map.set(flag as usize, value);
        self.flags = map.into_value();
        previous
    }

    /// Return whether the user account is initialized.
    pub fn is_initialized(&self) -> bool {
        self.flag(UserFlag::Initialized)
    }

    /// Initialize.
    pub(crate) fn init(&mut self, store: &Pubkey, owner: &Pubkey, bump: u8) -> Result<()> {
        require!(
            !self.flag(UserFlag::Initialized),
            CoreError::UserAccountHasBeenInitialized
        );
        self.set_flag(UserFlag::Initialized, true);

        self.bump = bump;
        self.owner = *owner;
        self.store = *store;

        Ok(())
    }

    /// Get User Account space.
    pub fn space(_version: u8) -> usize {
        core::mem::size_of::<Self>()
    }

    /// Get referral.
    pub fn referral(&self) -> &Referral {
        &self.referral
    }
}

impl Seed for UserHeader {
    const SEED: &'static [u8] = b"user";
}

/// User flags.
#[repr(u8)]
#[non_exhaustive]
pub enum UserFlag {
    /// Is initialized.
    Initialized,
}

impl UserFlag {
    /// Max flags.
    pub const MAX_FLAGS: usize = 8;
}

type UserFlagMap = bitmaps::Bitmap<{ UserFlag::MAX_FLAGS }>;
type UserFlagValue = u8;

/// Referral Code Bytes.
pub type ReferralCodeBytes = [u8; 4];

/// Referral.
#[zero_copy]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Referral {
    /// The User account address of the referrer.
    ///
    /// `Pubkey::default()` means no referrer.
    pub(crate) referrer: Pubkey,
    /// Referral Code Address.
    pub(crate) code: Pubkey,
    /// Number of referee.
    referee_count: u128,
    reserved: [u8; 64],
}

impl Referral {
    pub(crate) fn set_code(&mut self, code: &Pubkey) -> Result<()> {
        require_eq!(
            self.code,
            Pubkey::default(),
            CoreError::ReferralCodeHasBeenSet
        );

        self.code = *code;

        Ok(())
    }

    pub(crate) fn set_referrer(&mut self, referrer: &AccountLoader<UserHeader>) -> Result<()> {
        require_eq!(
            self.referrer,
            Pubkey::default(),
            CoreError::ReferrerHasBeenSet,
        );

        self.referrer = referrer.key();

        {
            let mut referrer = referrer.load_mut()?;
            referrer.referral.referee_count = referrer.referral.referee_count.saturating_add(1);
        }
        Ok(())
    }

    /// Get the user account address of the referrer.
    pub fn referrer(&self) -> Option<&Pubkey> {
        if self.referrer == Pubkey::default() {
            None
        } else {
            Some(&self.referrer)
        }
    }
}

/// Referral Code.
#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ReferralCode {
    /// Bump.
    pub(crate) bump: u8,
    /// Code bytes.
    pub code: ReferralCodeBytes,
    /// Store.
    pub store: Pubkey,
    /// Owner.
    pub owner: Pubkey,
}

impl ReferralCode {
    /// Init Space.
    pub const INIT_SPACE: usize = core::mem::size_of::<Self>();
}

impl Seed for ReferralCode {
    const SEED: &'static [u8] = b"referral_code";
}

/// GT State.
#[zero_copy]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct GTState {
    pub(crate) minted: u64,
    pub(crate) last_minted_at: i64,
    pub(crate) traded_value: u128,
    pub(crate) minted_value: u128,
    reserved: [u8; 64],
}

impl GTState {
    /// Get traded value.
    pub fn traded_value(&self) -> u128 {
        self.traded_value
    }

    /// Get minted value.
    pub fn minted_value(&self) -> u128 {
        self.minted_value
    }
}
