use crate::prelude::*;
use core::cmp::Ordering;
use gstd::{BlockCount, BlockNumber, Config, exec};

/// Type of wait locks.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Lock {
    WaitFor(BlockNumber),
    WaitUpTo(BlockNumber),
}

impl Lock {
    /// Wait for
    pub fn exactly(b: BlockCount) -> Self {
        let current = exec::block_height();
        Self::WaitFor(current.saturating_add(b))
    }

    /// Wait up to
    pub fn up_to(b: BlockCount) -> Self {
        let current = exec::block_height();
        Self::WaitUpTo(current.saturating_add(b))
    }

    /// Call wait functions by the lock type.
    pub fn wait(&self, now: BlockNumber) {
        match &self {
            Lock::WaitFor(d) => exec::wait_for(d.checked_sub(now).expect("Checked in `crate::gstd::async_runtime::message_loop`")),
            Lock::WaitUpTo(d) => exec::wait_up_to(d.checked_sub(now).expect("Checked in `crate::gstd::async_runtime::message_loop`")),
        }
    }

    /// Gets the deadline of the current lock.
    pub fn deadline(&self) -> BlockNumber {
        match &self {
            Lock::WaitFor(d) => *d,
            Lock::WaitUpTo(d) => *d,
        }
    }
}

impl PartialOrd for Lock {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Lock {
    fn cmp(&self, other: &Self) -> Ordering {
        self.deadline().cmp(&other.deadline())
    }
}

impl Default for Lock {
    fn default() -> Self {
        Lock::up_to(Config::wait_up_to())
    }
}
