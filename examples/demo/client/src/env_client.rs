use sails_rs::{
    client::{Actor, Deployment, GearEnv, PendingCall, PendingCtor, Service},
    prelude::*,
};

pub trait DemoCtors {
    type Env: GearEnv;

    fn default(self) -> PendingCtor<Self::Env, DemoProgram, io::Default>;
    fn new(
        self,
        counter: Option<u32>,
        dog_position: Option<(i32, i32)>,
    ) -> PendingCtor<Self::Env, DemoProgram, io::New>;
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
    ) -> Deployment<E, DemoProgram> {
        Deployment::new(env, code_id, salt)
    }

    pub fn client<E: GearEnv>(env: E, program_id: ActorId) -> Actor<DemoProgram, E> {
        Actor::new(program_id, env)
    }
}

impl<E: GearEnv> DemoCtors for Deployment<E, DemoProgram> {
    type Env = E;

    fn default(self) -> PendingCtor<Self::Env, DemoProgram, io::Default> {
        self.pending_ctor(())
    }

    fn new(
        self,
        counter: Option<u32>,
        dog_position: Option<(i32, i32)>,
    ) -> PendingCtor<Self::Env, DemoProgram, io::New> {
        self.pending_ctor((counter, dog_position))
    }
}

impl<E: GearEnv> Demo for Actor<DemoProgram, E> {
    type Env = E;

    fn counter(&self) -> Service<CounterImpl, Self::Env> {
        self.service("Counter")
    }
}

pub mod io {
    use super::*;
    use sails_rs::calls::ActionIo;
    pub struct Default(());
    impl Default {
        #[allow(dead_code)]
        pub fn encode_call() -> Vec<u8> {
            <Default as ActionIo>::encode_call(&())
        }
    }
    impl ActionIo for Default {
        const ROUTE: &'static [u8] = &[28, 68, 101, 102, 97, 117, 108, 116];
        type Params = ();
        type Reply = ();
    }
    pub struct New(());
    impl New {
        #[allow(dead_code)]
        pub fn encode_call(counter: Option<u32>, dog_position: Option<(i32, i32)>) -> Vec<u8> {
            <New as ActionIo>::encode_call(&(counter, dog_position))
        }
    }
    impl ActionIo for New {
        const ROUTE: &'static [u8] = &[12, 78, 101, 119];
        type Params = (Option<u32>, Option<(i32, i32)>);
        type Reply = ();
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
