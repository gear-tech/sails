use crate::prelude::*;
use core::cmp::Ordering;
use gstd::{BlockCount, BlockNumber, Config, exec};

/// Type of wait locks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Lock {
    deadline: BlockNumber,
    ty: WaitType,
}

/// Wait types.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum WaitType {
    Exactly,
    #[default]
    UpTo,
}

impl Lock {
    /// Wait for
    pub fn exactly(b: BlockCount) -> Self {
        let current = Syscall::block_height();
        Self {
            deadline: current.saturating_add(b),
            ty: WaitType::Exactly,
        }
    }

    /// Wait up to
    pub fn up_to(b: BlockCount) -> Self {
        let current = Syscall::block_height();
        Self {
            deadline: current.saturating_add(b),
            ty: WaitType::UpTo,
        }
    }

    /// Gets the deadline of the current lock.
    pub fn deadline(&self) -> BlockNumber {
        self.deadline
    }

    /// Gets the duration from current [`Syscall::block_height()`].
    pub fn duration(&self) -> Option<BlockCount> {
        let current = Syscall::block_height();
        self.deadline.checked_sub(current)
    }

    pub fn wait_type(&self) -> WaitType {
        self.ty
    }

    /// Call wait functions by the lock type.
    pub fn wait(&self, now: BlockNumber) {
        let duration = self
            .deadline
            .checked_sub(now)
            .expect("Checked in `crate::gstd::async_runtime::message_loop`");
        match self.ty {
            WaitType::Exactly => exec::wait_for(duration),
            WaitType::UpTo => exec::wait_up_to(duration),
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
        let mut ord = self.deadline().cmp(&other.deadline());
        if ord == Ordering::Equal {
            ord = match self.wait_type() {
                WaitType::Exactly => Ordering::Greater,
                WaitType::UpTo => Ordering::Less,
            }
        }
        ord
    }
}

impl Default for Lock {
    fn default() -> Self {
        Lock::up_to(Config::wait_up_to())
    }
}
