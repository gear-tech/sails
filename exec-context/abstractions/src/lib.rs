#![no_std]

pub trait ExecContext {
    type ActorId;

    fn actor_id(&self) -> &Self::ActorId;
}
