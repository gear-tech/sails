#![no_std]

// Incorporate code generated based on the [IDL](/examples/demo/wasm/demo.idl) file
include!("demo_client.rs");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_module_encode() {
        use this_that::*;

        let bytes = io::DoThat::encode_params_with_prefix(
            "ThisThat",
            DoThatParam {
                p1: NonZeroU32::MAX,
                p2: 123.into(),
                p3: ManyVariants::One,
            },
        );

        assert_eq!(
            bytes,
            vec![
                32, 84, 104, 105, 115, 84, 104, 97, 116, // ThisThat
                24, 68, 111, 84, 104, 97, 116, // DoThat
                255, 255, 255, 255, // p1
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 123, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, // p2
                0, // p3
            ]
        );
    }

    #[test]
    fn test_io_module_decode_reply() {
        use this_that::*;

        let bytes = vec![
            32, 84, 104, 105, 115, 84, 104, 97, 116, // ThisThat
            24, 68, 111, 84, 104, 97, 116, // DoThat
            0,   // Ok
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 123, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, // 123
            255, 255, 255, 255, // NonZeroU32::MAX
            0,   // ManyVariantsReply::One
        ];

        let reply: Result<(ActorId, NonZeroU32, ManyVariantsReply), (String,)> =
            io::DoThat::decode_reply_with_prefix("ThisThat", bytes).unwrap();

        assert_eq!(
            reply,
            Ok((ActorId::from(123), NonZeroU32::MAX, ManyVariantsReply::One))
        );
    }
}
