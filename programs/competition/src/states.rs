use anchor_lang::prelude::*;

/// The expected program ID of the caller.
pub const CALLER_PROGRAM_ID: Pubkey = gmsol_programs::gmsol_store::ID_CONST;

/// The seed for [`Competition`] account.
#[constant]
pub const COMPETITION_SEED: &[u8] = b"competition";

/// The seed for [`Participant`] account.
#[constant]
pub const PARTICIPANT_SEED: &[u8] = b"participant";

/// The maximum number of leaderboard entries kept on chain.
#[constant]
pub const MAX_LEADERBOARD_LEN: u8 = 5;

/// A single leaderboard record.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Default, PartialEq, Eq, InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct LeaderEntry {
    /// The trader address.
    pub address: Pubkey,
    /// The cumulative traded volume.
    pub volume: u128,
}

/// The global competition data.
#[account]
#[derive(InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Competition {
    /// Bump seed.
    pub bump: u8,
    /// The authority of this competition.
    pub authority: Pubkey,
    /// The competition start timestamp.
    pub start_time: i64,
    /// The competition end timestamp.
    pub end_time: i64,
    /// The fixed-length leaderboard.
    #[max_len(MAX_LEADERBOARD_LEN)]
    pub leaderboard: Vec<LeaderEntry>,
    /// Volume threshold in USD.
    pub volume_threshold: u128,
    /// Time extension in seconds.
    pub extension_duration: i64,
    /// Maximum time extension in seconds.
    pub extension_cap: i64,
    /// Address that triggered time extension.
    pub extension_triggerer: Option<Pubkey>,
    /// Whether to only count volume from position increases.
    pub only_count_increase: bool,
    /// Time window in seconds for merging volumes from the same trader.
    pub volume_merge_window: i64,
}

impl Competition {
    /// Returns whether the competition is ongoing at the given time.
    pub fn is_ongoing(&self, now: i64) -> bool {
        now >= self.start_time && now <= self.end_time
    }
}

/// The per-trader statistics.
#[account]
#[derive(InitSpace)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Participant {
    /// Bump seed.
    pub bump: u8,
    /// The competition account this entry belongs to.
    pub competition: Pubkey,
    /// The trader address.
    pub trader: Pubkey,
    /// The cumulative traded volume.
    pub volume: u128,
    /// The last update timestamp.
    pub last_updated_at: i64,
    /// The merged volume within the time window.
    pub merged_volume: u128,
}
