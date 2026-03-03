#![no_std]

// Incorporate code generated based on the [IDL](/examples/demo/wasm/demo.idl) file
include!("demo_client.rs");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_module_encode() {
        use this_that::*;

        // Use the new simplified encode_call method
        // It automatically uses the InterfaceId from the service module
        let bytes = io::DoThat::encode_call(
            DemoClientProgram::ROUTE_ID_THIS_THAT,
            DoThatParam {
                p1: NonZeroU32::MAX,
                p2: 123.into(),
                p3: ManyVariants::One,
            },
        );

        let mut expected = vec![
            0x47, 0x4D, 0x01, 0x10, // Magic, Version, Header Length
            68, 91, 237, 110, 251, 232, 230, 221, // Interface ID
            0, 0, // Entry ID
            5, // Route Index
            0, // Reserved
        ];
        expected.extend_from_slice(&[
            255, 255, 255, 255, // p1
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 123, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, // p2
            0, // p3
        ]);

        assert_eq!(bytes, expected);
    }

    #[test]
    fn test_io_module_decode_reply() {
        use this_that::*;

        // We don't need manual InterfaceId anymore, it's inside decode_reply
        let mut bytes = vec![
            0x47, 0x4D, 0x01, 0x10, // Magic, Version, Header Length
            68, 91, 237, 110, 251, 232, 230, 221, // Interface ID
            0, 0, // Entry ID
            0, // Route Index
            0, // Reserved
        ];
        bytes.extend_from_slice(&[
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 123, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, // 123
            255, 255, 255, 255, // NonZeroU32::MAX
            0,   // ManyVariantsReply::One
        ]);

        // Use the simplified method
        let reply: Result<(ActorId, NonZeroU32, ManyVariantsReply), (String,)> =
            io::DoThat::decode_reply(0, bytes).unwrap();

        assert_eq!(
            reply,
            Ok((ActorId::from(123), NonZeroU32::MAX, ManyVariantsReply::One))
        );
    }
}
