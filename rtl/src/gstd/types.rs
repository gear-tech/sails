use crate::types::{ActorId, CodeId, MessageId};
use gstd::{ActorId as GStdActorId, CodeId as GStdCodeId, MessageId as GStdMessageId};

impl From<ActorId> for GStdActorId {
    fn from(actor_id: ActorId) -> Self {
        GStdActorId::new(*actor_id.as_ref())
    }
}

impl From<GStdActorId> for ActorId {
    fn from(actor_id: GStdActorId) -> Self {
        ActorId::from(Into::<[u8; 32]>::into(actor_id))
    }
}

impl From<MessageId> for GStdMessageId {
    fn from(message_id: MessageId) -> Self {
        GStdMessageId::new(*message_id.as_ref())
    }
}

impl From<GStdMessageId> for MessageId {
    fn from(message_id: GStdMessageId) -> Self {
        MessageId::from(Into::<[u8; 32]>::into(message_id))
    }
}

impl From<CodeId> for GStdCodeId {
    fn from(code_id: CodeId) -> Self {
        GStdCodeId::new(*code_id.as_ref())
    }
}

impl From<GStdCodeId> for CodeId {
    fn from(code_id: GStdCodeId) -> Self {
        CodeId::from(Into::<[u8; 32]>::into(code_id))
    }
}
