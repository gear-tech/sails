use crate::types::ActorId;
use gstd::ActorId as GStdActorId;

impl From<ActorId> for GStdActorId {
    fn from(actor_id: ActorId) -> Self {
        GStdActorId::new(*actor_id.as_ref())
    }
}
