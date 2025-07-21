mod bls381;
mod eth_bridge;
mod proxy;
mod staking;

pub use bls381::*;
pub use eth_bridge::*;
pub use proxy::*;
pub use staking::*;

use crate::calls::Remoting;

pub trait BuiltinsRemoting: Remoting {}

#[cfg(feature = "gstd")]
impl BuiltinsRemoting for crate::gstd::calls::GStdRemoting {}

#[cfg(feature = "gclient")]
#[cfg(not(target_arch = "wasm32"))]
impl BuiltinsRemoting for crate::gclient::calls::GClientRemoting {}

macro_rules! builtin_action {
    (
        $enum_name:ident,
        $builtin_name:ident,
        $variant:ident $( { $($field:ident : $field_type:ty),* $(,)? } )?
        $( => $reply_ty:ty )?
    ) => {
        pub struct $variant(());

        impl ActionIo for $variant {
            const ROUTE: &'static [u8] = b"";
            type Params = $enum_name;
            type Reply = builtin_action!(@reply_type $( $reply_ty )?);

            fn encode_call(value: &$enum_name) -> Vec<u8> {
                builtin_action!(@match_variant value, $enum_name, $variant $(, { $($field),* })?);
                value.encode()
            }

            fn decode_reply(payload: impl AsRef<[u8]>) -> Result<Self::Reply> {
                let value = payload.as_ref();
                builtin_action!(@decode_body value $( $reply_ty )?)
            }
        }

        impl<R: BuiltinsRemoting + Clone> $builtin_name<R> {
            paste::item! {
                pub fn [<$variant:snake>](
                    &self
                    $(, $($field : $field_type),* )?
                ) -> impl Call<Output = <$variant as ActionIo>::Reply, Args = R::Args> {
                    let request = $enum_name::$variant $( { $($field),* } )?;
                    RemotingAction::<_, $variant>::new(self.remoting.clone(), request)
                }
            }
        }
    };

    // Helper arm: extract reply type or default
    (@reply_type $reply_ty:ty) => { $reply_ty };
    (@reply_type) => { Vec<u8> };

    // Helper: Match variant with fields
    (@match_variant $value:ident, $enum_name:ident, $variant:ident, { $($field:ident),* }) => {
        if !matches!($value, $enum_name::$variant { .. }) {
            panic!(
                "internal error: invalid param received. Expected `{}::{}`, received: {:?}",
                stringify!($enum_name), stringify!($variant), $value
            );
        }
    };

    // Helper: Match unit variant
    (@match_variant $value:ident, $enum_name:ident, $variant:ident) => {
        if !matches!($value, $enum_name::$variant) {
            panic!(
                "internal error: invalid param received. Expected `{}::{}`, received: {:?}",
                stringify!($enum_name), stringify!($variant), $value
            );
        }
    };

        // Helper arm: decode body based on presence of reply type
    (@decode_body $value:ident $reply_ty:ty) => {{
        let mut val = $value;
        <$reply_ty as Decode>::decode(&mut val).map_err(Error::Codec)
    }};
    (@decode_body $value:ident) => {{
        if !$value.is_empty() {
            // todo [sab] change to error type
            panic!(
                "internal error: expected empty reply for unit variant, received: {:?}",
                $value
            );
        }
        Ok(Default::default())
    }};
}

pub(crate) use builtin_action;
