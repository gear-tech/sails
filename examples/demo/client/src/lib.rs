#![no_std]

// Incorporate code generated based on the [IDL](/examples/demo/wasm/demo.idl) file
include!("demo_client.rs");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_module_encode() {
        use this_that::*;

        let interface_id = InterfaceId::from_bytes_8([68, 91, 237, 110, 251, 232, 230, 221]);
        let bytes = io::DoThat::encode_params_with_header(
            interface_id,
            0,
            DoThatParam {
                p1: NonZeroU32::MAX,
                p2: 123.into(),
                p3: ManyVariants::One,
            },
        );

        let mut expected = vec![
            0x47, 0x4D, 0x01, 0x0B, // Magic, Version, Header Length
            68, 91, 237, 110, 251, 232, 230, 221, // Interface ID
            0, 0, // Entry ID
            0, // Route Index
            0, // Reserved
        ];
        expected.extend_from_slice(&[
            255, 255, 255, 255, // p1
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 123, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, // p2
            0, // p3
        ]);

        assert_eq!(bytes, expected);
    }

    #[test]
    fn test_io_module_decode_reply() {
        use this_that::*;

        let interface_id = InterfaceId::from_bytes_8([68, 91, 237, 110, 251, 232, 230, 221]);
        let mut bytes = vec![
            0x47, 0x4D, 0x01, 0x0B, // Magic, Version, Header Length
            68, 91, 237, 110, 251, 232, 230, 221, // Interface ID
            0, 0, // Entry ID
            0, // Route Index
            0, // Reserved
        ];
        bytes.extend_from_slice(&[
            0, // Ok
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 123, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, // 123
            255, 255, 255, 255, // NonZeroU32::MAX
            0, // ManyVariantsReply::One
        ]);

        let reply: Result<(ActorId, NonZeroU32, ManyVariantsReply), (String,)> =
            io::DoThat::decode_reply_with_header(interface_id, 0, bytes).unwrap();

        assert_eq!(
            reply,
            Ok((ActorId::from(123), NonZeroU32::MAX, ManyVariantsReply::One))
        );
    }
}
