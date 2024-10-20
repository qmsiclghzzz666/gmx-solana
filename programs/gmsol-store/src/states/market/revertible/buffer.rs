use anchor_lang::prelude::*;
use gmsol_model::{ClockKind, PoolKind};
use strum::IntoEnumIterator;

use crate::states::{
    market::{Clocks, State},
    OtherState, Pool,
};

#[account(zero_copy)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub(crate) struct RevertibleBuffer {
    rev: u64,
    padding: [u8; 8],
    state: State,
}

impl RevertibleBuffer {
    pub(crate) fn init(&mut self) {
        self.rev = 1;
    }

    pub(super) fn pool<'a>(&'a self, kind: PoolKind, storage: &'a State) -> Option<&'a Pool> {
        let pool = self.state.pools.get(kind)?;
        Some(pool.cache_get_with(self.rev, || storage.pools.get(kind).expect("must exist")))
    }

    pub(super) fn pool_mut(&mut self, kind: PoolKind, storage: &State) -> Option<&mut Pool> {
        let pool = self.state.pools.get_mut(kind)?;
        Some(pool.cache_get_mut_with(self.rev, || *storage.pools.get(kind).expect("must exist")))
    }

    pub(super) fn clocks<'a>(&'a self, storage: &'a State) -> &Clocks {
        self.state
            .clocks
            .cache_get_with(self.rev, || &storage.clocks)
    }

    pub(super) fn clocks_mut(&mut self, storage: &State) -> &mut Clocks {
        self.state
            .clocks
            .cache_get_mut_with(self.rev, || storage.clocks)
    }

    pub(super) fn other<'a>(&'a self, storage: &'a State) -> &OtherState {
        self.state.other.cache_get_with(self.rev, || &storage.other)
    }

    pub(super) fn other_mut(&mut self, storage: &State) -> &mut OtherState {
        self.state
            .other
            .cache_get_mut_with(self.rev, || storage.other)
    }

    pub(super) fn commit_to_storage(&mut self, storage: &mut State) {
        let state = &mut self.state;

        // Commit pools.
        for kind in PoolKind::iter() {
            let Some(pool) = state.pools.get_mut(kind) else {
                continue;
            };
            if pool.is_dirty() {
                let target = storage.pools.get_mut(kind).expect("must exist");
                pool.clear_dirty();
                *target = *pool;
            }
        }

        // Commit clocks.
        if state.clocks.is_dirty() {
            state.clocks.clear_dirty();
            storage.clocks = state.clocks;
        }

        // Commit other state.
        if state.other.is_dirty() {
            state.other.clear_dirty();
            storage.other = state.other;
        }

        self.rev = self.rev.checked_add(1).expect("rev overflow");
    }
}

const DIRTY_VALUE: u8 = 1;

pub(super) trait Cache {
    fn set_dirty(&mut self);

    fn clear_dirty(&mut self);

    fn is_dirty(&self) -> bool;

    fn rev(&self) -> u64;

    fn set_rev(&mut self, rev: u64) -> u64;

    fn cache_get_with<'a>(&'a self, rev: u64, f: impl FnOnce() -> &'a Self) -> &'a Self {
        if self.rev() == rev {
            self
        } else {
            f()
        }
    }

    fn cache_get_mut_with(&mut self, rev: u64, f: impl FnOnce() -> Self) -> &mut Self
    where
        Self: Sized,
    {
        if self.rev() != rev {
            *self = f();
            self.set_rev(rev);
        }
        self.set_dirty();
        self
    }
}

macro_rules! impl_cache {
    ($cache:ty) => {
        impl Cache for $cache {
            fn set_dirty(&mut self) {
                self.dirty = DIRTY_VALUE;
            }

            fn clear_dirty(&mut self) {
                self.dirty = 0;
            }

            fn is_dirty(&self) -> bool {
                self.dirty == DIRTY_VALUE
            }

            fn rev(&self) -> u64 {
                self.rev
            }

            fn set_rev(&mut self, mut rev: u64) -> u64 {
                std::mem::swap(&mut rev, &mut self.rev);
                rev
            }
        }
    };
}

impl_cache!(Clocks);
impl_cache!(Pool);
impl_cache!(OtherState);
