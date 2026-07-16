//! On `ethexe`, programs are deployed via the L1, so `GstdEnv` must NOT implement
//! `EnvWithCtor` — creating a program from within a program has to be a
//! compile-time error rather than a runtime panic. Generated client constructors
//! require `E: EnvWithCtor`, so this assertion stands in for `GstdEnv.deploy(..).new(..)`.

use sails::client::{EnvWithCtor, GstdEnv};

fn requires_env_with_ctor<E: EnvWithCtor>() {}

fn main() {
    requires_env_with_ctor::<GstdEnv>();
}
