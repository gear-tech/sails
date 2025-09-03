use sails_rs::{client::*, prelude::*};

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

    fn counter(&self) -> Service<Self::Env, counter::CounterImpl>;
}

pub struct DemoProgram;

impl Program for DemoProgram {}

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

impl<E: GearEnv> Demo for Actor<E, DemoProgram> {
    type Env = E;

    fn counter(&self) -> Service<Self::Env, counter::CounterImpl> {
        self.service("Counter")
    }
}

pub mod io {
    use super::*;
    use sails_rs::client::{CallEncodeDecode, Route};
    sails_rs::io_struct_impl!(Default () -> ());
    sails_rs::io_struct_impl!(New (counter: Option<u32>, dog_position: Option<(i32, i32),>) -> ());
}

/// Counter Service
pub mod counter {
    use super::*;
    pub trait Counter {
        type Env: GearEnv;

        fn add(&mut self, value: u32) -> PendingCall<Self::Env, io::Add>;
        fn sub(&mut self, value: u32) -> PendingCall<Self::Env, io::Sub>;
        fn value(&self) -> PendingCall<Self::Env, io::Value>;
    }

    pub struct CounterImpl;

    impl<E: GearEnv> Counter for Service<E, CounterImpl> {
        type Env = E;

        fn add(&mut self, value: u32) -> PendingCall<Self::Env, io::Add> {
            self.pending_call((value,))
        }

        fn sub(&mut self, value: u32) -> PendingCall<Self::Env, io::Sub> {
            self.pending_call((value,))
        }

        fn value(&self) -> PendingCall<Self::Env, io::Value> {
            self.pending_call(())
        }
    }

    pub mod io {
        use super::*;
        use sails_rs::client::{CallEncodeDecode, Route};
        sails_rs::io_struct_impl!(Add (value: u32) -> u32);
        sails_rs::io_struct_impl!(Sub (value: u32) -> u32);
        sails_rs::io_struct_impl!(Value () -> u32);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub mod events {
        use super::*;
        #[derive(PartialEq, Debug, Encode, Decode)]
        #[codec(crate = sails_rs::scale_codec)]
        pub enum CounterEvents {
            /// Emitted when a new value is added to the counter
            Added(u32),
            /// Emitted when a value is subtracted from the counter
            Subtracted(u32),
        }
        impl EventDecode for CounterEvents {
            const EVENT_NAMES: &'static [Route] = &["Added", "Subtracted"];
        }

        impl ServiceEvent for CounterImpl {
            type Event = CounterEvents;
        }
    }
}

#[cfg(feature = "with_mocks")]
#[cfg(not(target_arch = "wasm32"))]
pub mod mockall {
    use super::*;
    use sails_rs::mockall::*;
    mock! {
        pub Counter {}
        #[allow(refining_impl_trait)]
        #[allow(clippy::type_complexity)]
        impl counter::Counter for Counter {
            type Env = GstdEnv;
            fn add (&mut self, value: u32) -> PendingCall<GstdEnv, counter::io::Add>;
            fn sub (&mut self, value: u32) -> PendingCall<GstdEnv, counter::io::Sub>;
            fn value (& self, ) -> PendingCall<GstdEnv, counter::io::Value>;
        }
    }
}
