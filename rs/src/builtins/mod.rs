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

// Creates an action for a provided builtin-in request variant and implements the `ActionIo` trait for it.
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
    (@reply_type) => { () };

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
            panic!(
                "internal error: expected empty reply for unit variant, received: {:?}",
                $value
            );
        }
        Ok(())
    }};
}

pub(crate) use builtin_action;

#[cfg(test)]
mod test_utils {
    macro_rules! assert_action_codec {
        // Check that action is encoded/decoded without any routes.
        (
            $req_enum_name:ident,
            $req_variant:ident $({ $($req_field:ident : $req_value:expr),* $(,)? })?,
            $resp_enum_name:ident,
            $response_variant:ident $response_body:tt
        ) => {{
            let req = $req_enum_name::$req_variant $({ $($req_field: $req_value),* })?;
            let resp = $crate::builtins::test_utils::assert_action_codec!(@build_response $resp_enum_name, $response_variant $response_body);
            let encoded_action = $req_variant::encode_call(&req);
            assert_eq!(req.encode(), encoded_action);
            let decoded_resp = $req_variant::decode_reply(resp.encode()).unwrap();
            assert_eq!(resp, decoded_resp);
        }};

        // Check that value is encoded without any routes, and decoded into `()`.
        (
            $req_enum_name:ident,
            $req_variant:ident $({ $($req_field:ident : $req_value:expr),* $(,)? })?
        ) => {{
            let req = $req_enum_name::$req_variant $({ $($req_field: $req_value),* })?;
            let encoded_action = $req_variant::encode_call(&req);
            assert_eq!(req.encode(), encoded_action);
            assert_eq!(<$req_variant as ActionIo>::Reply::type_info(), <()>::type_info());
        }};

        // Helper: Build response with tuple syntax
        (@build_response $resp_enum_name:ident, $response_variant:ident ($response_value:expr)) => {
            $resp_enum_name::$response_variant($response_value)
        };

        // Helper: Build response with struct syntax
        (@build_response $resp_enum_name:ident, $response_variant:ident { $($resp_field:ident : $resp_value:expr),* $(,)? }) => {
            $resp_enum_name::$response_variant { $($resp_field: $resp_value),* }
        };
    }

    pub(crate) use assert_action_codec;
}
