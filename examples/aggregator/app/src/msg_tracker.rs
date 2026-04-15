use sails_rs::collections::BTreeMap;
use sails_rs::prelude::*;

#[sails_type]
#[derive(Clone, PartialEq, Debug)]
pub enum OpStatus {
    Started,
    Step1,
    Step2,
    Finalized,
}

pub struct MsgTracker {
    message_info: BTreeMap<MessageId, OpStatus>,
}

impl MsgTracker {
    pub fn new() -> Self {
        Self {
            message_info: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, msg_id: MessageId, status: OpStatus) {
        self.message_info.insert(msg_id, status);
    }

    pub fn update_status(&mut self, msg_id: MessageId, status: OpStatus) -> bool {
        if let Some(s) = self.message_info.get_mut(&msg_id) {
            *s = status;
            true
        } else {
            false
        }
    }

    pub fn get_status(&self, msg_id: &MessageId) -> Option<OpStatus> {
        self.message_info.get(msg_id).cloned()
    }

    pub fn get_statuses(&self) -> Vec<(MessageId, OpStatus)> {
        self.message_info
            .iter()
            .map(|(&id, s)| (id, s.clone()))
            .collect()
    }
}

impl Default for MsgTracker {
    fn default() -> Self {
        Self::new()
    }
}
