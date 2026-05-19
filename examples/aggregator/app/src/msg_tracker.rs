use sails_rs::{collections::BTreeMap, prelude::*};
use sails_storage::FixedOpenAddressMap;

const FIXED_TRACKER_CAPACITY: usize = 2048;

#[sails_type]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OpStatus {
    Started,
    Step1,
    Step2,
    Finalized,
}

#[sails_type]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TrackerBackend {
    BTree,
    SailsFixed,
}

#[sails_type]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TrackerOp {
    InsertFresh,
    UpdateExisting,
    ReadExisting,
    ListStatuses,
}

#[sails_type]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct TrackerBenchResult {
    pub len: u32,
    pub status: Option<OpStatus>,
    pub existed: bool,
}

pub struct MsgTracker {
    storage: TrackerStorage,
}

enum TrackerStorage {
    BTree(BTreeMap<MessageId, OpStatus>),
    SailsFixed(FixedOpenAddressMap<32, 1, FIXED_TRACKER_CAPACITY>),
}

impl MsgTracker {
    pub fn new(backend: TrackerBackend) -> Self {
        Self {
            storage: match backend {
                TrackerBackend::BTree => TrackerStorage::BTree(BTreeMap::new()),
                TrackerBackend::SailsFixed => {
                    TrackerStorage::SailsFixed(FixedOpenAddressMap::new())
                }
            },
        }
    }

    pub fn backend(&self) -> TrackerBackend {
        match self.storage {
            TrackerStorage::BTree(_) => TrackerBackend::BTree,
            TrackerStorage::SailsFixed(_) => TrackerBackend::SailsFixed,
        }
    }

    pub fn len(&self) -> u32 {
        match &self.storage {
            TrackerStorage::BTree(map) => map.len() as u32,
            TrackerStorage::SailsFixed(map) => map.len() as u32,
        }
    }

    pub fn clear(&mut self) {
        let backend = self.backend();
        *self = Self::new(backend);
    }

    pub fn insert(&mut self, msg_id: MessageId, status: OpStatus) {
        self.insert_inner(msg_id, status);
    }

    pub fn insert_inner(&mut self, msg_id: MessageId, status: OpStatus) -> Option<OpStatus> {
        match &mut self.storage {
            TrackerStorage::BTree(map) => map.insert(msg_id, status),
            TrackerStorage::SailsFixed(map) => map
                .insert(message_id_key(msg_id), status_value(status))
                .expect("fixed tracker capacity exceeded")
                .map(value_status),
        }
    }

    pub fn update_status(&mut self, msg_id: MessageId, status: OpStatus) -> bool {
        match &mut self.storage {
            TrackerStorage::BTree(map) => {
                if let Some(s) = map.get_mut(&msg_id) {
                    *s = status;
                    true
                } else {
                    false
                }
            }
            TrackerStorage::SailsFixed(map) => {
                let key = message_id_key(msg_id);
                if map
                    .get(&key)
                    .expect("fixed tracker lookup failed")
                    .is_some()
                {
                    map.insert(key, status_value(status))
                        .expect("fixed tracker update failed");
                    true
                } else {
                    false
                }
            }
        }
    }

    pub fn get_status(&self, msg_id: &MessageId) -> Option<OpStatus> {
        match &self.storage {
            TrackerStorage::BTree(map) => map.get(msg_id).copied(),
            TrackerStorage::SailsFixed(map) => map
                .get(&message_id_key(*msg_id))
                .expect("fixed tracker lookup failed")
                .map(value_status),
        }
    }

    pub fn get_statuses(&self) -> Vec<(MessageId, OpStatus)> {
        match &self.storage {
            TrackerStorage::BTree(map) => map.iter().map(|(&id, &status)| (id, status)).collect(),
            TrackerStorage::SailsFixed(map) => {
                let mut statuses = map
                    .entries()
                    .map(|(key, status)| (message_id_from_key(key), value_status(status)))
                    .collect::<Vec<_>>();
                statuses.sort_unstable_by_key(|(id, _)| *id);
                statuses
            }
        }
    }
}

impl Default for MsgTracker {
    fn default() -> Self {
        Self::new(TrackerBackend::BTree)
    }
}

pub fn message_id_for_seed(seed: u32) -> MessageId {
    MessageId::from(seed as u64 + 1)
}

fn message_id_key(msg_id: MessageId) -> [u8; 32] {
    let mut key = [0u8; 32];
    key.copy_from_slice(msg_id.as_ref());
    key
}

fn message_id_from_key(key: [u8; 32]) -> MessageId {
    MessageId::from(key)
}

fn status_value(status: OpStatus) -> [u8; 1] {
    [match status {
        OpStatus::Started => 0,
        OpStatus::Step1 => 1,
        OpStatus::Step2 => 2,
        OpStatus::Finalized => 3,
    }]
}

fn value_status(value: [u8; 1]) -> OpStatus {
    match value[0] {
        0 => OpStatus::Started,
        1 => OpStatus::Step1,
        2 => OpStatus::Step2,
        3 => OpStatus::Finalized,
        _ => panic!("invalid tracker status value"),
    }
}

#[cfg(test)]
mod tests {
    extern crate std;

    use super::*;

    #[test]
    fn backends_match_representative_operations() {
        for backend in [TrackerBackend::BTree, TrackerBackend::SailsFixed] {
            let mut tracker = MsgTracker::new(backend);
            let first = message_id_for_seed(1);
            let second = message_id_for_seed(2);

            assert_eq!(tracker.insert_inner(first, OpStatus::Started), None);
            assert_eq!(tracker.insert_inner(second, OpStatus::Step1), None);
            assert_eq!(
                tracker.insert_inner(first, OpStatus::Step2),
                Some(OpStatus::Started)
            );
            assert_eq!(tracker.get_status(&first), Some(OpStatus::Step2));
            assert!(tracker.update_status(second, OpStatus::Finalized));
            assert_eq!(
                tracker.get_statuses(),
                vec![(first, OpStatus::Step2), (second, OpStatus::Finalized),]
            );
        }
    }
}
