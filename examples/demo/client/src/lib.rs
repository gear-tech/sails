#![no_std]

// Incorporate code generated based on the [IDL](/examples/demo/wasm/demo.idl) file
include!(concat!(env!("OUT_DIR"), "/demo_client.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use sails_rtl::{calls::*, prelude::*};

    #[test]
    fn test_io_module_encode() {
        let bytes = this_that::io::DoThat::encode_call(DoThatParam {
            p1: NonZeroU32::MAX,
            p2: 123.into(),
            p3: ManyVariants::One,
        });

        assert_eq!(
            bytes,
            vec![
                32, 84, 104, 105, 115, 84, 104, 97, 116, // ThisThat
                24, 68, 111, 84, 104, 97, 116, // DoThat
                255, 255, 255, 255, // p1
                123, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, // p2
                0, // p3
            ]
        );
    }

    #[test]
    fn test_io_module_decode_reply() {
        let bytes = vec![
            32, 84, 104, 105, 115, 84, 104, 97, 116, // ThisThat
            24, 68, 111, 84, 104, 97, 116, // DoThat
            0,   // Ok
            123, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, // 123
            255, 255, 255, 255, // NonZeroU32::MAX
        ];

        let reply: Result<(ActorId, NonZeroU32), (String,)> =
            this_that::io::DoThat::decode_reply(bytes).unwrap();

        assert_eq!(reply, Ok((ActorId::from(123), NonZeroU32::MAX)));
    }
}
