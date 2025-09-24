use crate::prelude::*;
use core::cmp::Ordering;
use gstd::{BlockCount, BlockNumber, Config, exec};

/// Type of wait locks.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LockType {
    WaitFor(BlockCount),
    WaitUpTo(BlockCount),
}

/// Wait lock
#[derive(Debug, PartialEq, Eq)]
pub struct Lock {
    /// The start block number of this lock.
    pub at: BlockNumber,
    /// The type of this lock.
    ty: LockType,
}

impl Lock {
    /// Wait for
    pub fn exactly(b: BlockCount) -> Self {
        // if b == 0 {
        //     return Err(Error::Gstd(UsageError::EmptyWaitDuration));
        // }

        Self {
            at: exec::block_height(),
            ty: LockType::WaitFor(b),
        }
    }

    /// Wait up to
    pub fn up_to(b: BlockCount) -> Self {
        // if b == 0 {
        //     return Err(Error::Gstd(UsageError::EmptyWaitDuration));
        // }

        Self {
            at: exec::block_height(),
            ty: LockType::WaitUpTo(b),
        }
    }

    /// Call wait functions by the lock type.
    pub fn wait(&self, now: BlockNumber) {
        if let Some(blocks) = self.deadline().checked_sub(now) {
            if blocks == 0 {
                unreachable!(
                    "Checked in `crate::msg::async::poll`, will trigger the timeout error automatically."
                );
            }

            match self.ty {
                LockType::WaitFor(_) => exec::wait_for(blocks),
                LockType::WaitUpTo(_) => exec::wait_up_to(blocks),
            }
        } else {
            unreachable!(
                "Checked in `crate::msg::async::poll`, will trigger the timeout error automatically."
            );
        }
    }

    /// Gets the deadline of the current lock.
    pub fn deadline(&self) -> BlockNumber {
        match &self.ty {
            LockType::WaitFor(d) | LockType::WaitUpTo(d) => self.at.saturating_add(*d),
        }
    }

    /// Check if this lock is timed out.
    pub fn timeout(&self, now: BlockNumber) -> Option<(BlockNumber, BlockNumber)> {
        let expected = self.deadline();
        (now >= expected).then(|| (expected, now))
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

impl Default for LockType {
    fn default() -> Self {
        LockType::WaitUpTo(Config::wait_up_to())
    }
}
