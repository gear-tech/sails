use sails_rs::{
    client::{Actor, Deployment, GearEnv, PendingCall, PendingCtor, Service},
    prelude::*,
};

pub trait DemoCtors {
    type Env: GearEnv;

    fn default(self) -> PendingCtor<Self::Env, DemoProgram>;
    fn new(
        self,
        counter: Option<u32>,
        dog_position: Option<(i32, i32)>,
    ) -> PendingCtor<Self::Env, DemoProgram>;
}

pub trait Demo {
    type Env: GearEnv;

    fn counter(&self) -> Service<CounterImpl, Self::Env>;
}

pub struct DemoProgram;

impl DemoProgram {
    pub fn deploy<E: GearEnv>(
        env: E,
        code_id: CodeId,
        salt: Vec<u8>,
    ) -> Deployment<DemoProgram, E> {
        Deployment::new(env, code_id, salt)
    }

    pub fn client<E: GearEnv>(env: E, program_id: ActorId) -> Actor<DemoProgram, E> {
        Actor::new(program_id, env)
    }
}

impl<E: GearEnv> DemoCtors for Deployment<DemoProgram, E> {
    type Env = E;

    fn default(self) -> PendingCtor<Self::Env, DemoProgram> {
        self.pending_ctor("Default", ())
    }

    fn new(
        self,
        counter: Option<u32>,
        dog_position: Option<(i32, i32)>,
    ) -> PendingCtor<Self::Env, DemoProgram> {
        self.pending_ctor("New", (counter, dog_position))
    }
}

impl<E: GearEnv> Demo for Actor<DemoProgram, E> {
    type Env = E;

    fn counter(&self) -> Service<CounterImpl, Self::Env> {
        self.service("Counter")
    }
}

/// Counter Service
pub trait Counter {
    type Env: GearEnv;

    fn add(&mut self, value: u32) -> PendingCall<Self::Env, u32>;
    fn sub(&mut self, value: u32) -> PendingCall<Self::Env, u32>;
    fn value(&self) -> PendingCall<Self::Env, u32>;
}

pub struct CounterImpl;

impl<E: GearEnv> Counter for Service<CounterImpl, E> {
    type Env = E;

    fn add(&mut self, value: u32) -> PendingCall<Self::Env, u32> {
        self.pending_call("Add", (value,))
    }

    fn sub(&mut self, value: u32) -> PendingCall<Self::Env, u32> {
        self.pending_call("Sub", (value,))
    }

    fn value(&self) -> PendingCall<Self::Env, u32> {
        self.pending_call("Value", ())
    }
}
