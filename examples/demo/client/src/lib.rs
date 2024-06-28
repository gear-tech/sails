#![no_std]

include!(concat!(env!("OUT_DIR"), "/demo_client.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_module_encode() {
        let bytes = this_that_io::DoThat::encode_call(DoThatParam {
            p1: u32::MAX,
            p2: "hello".to_string(),
            p3: ManyVariants::One,
        });

        assert_eq!(
            bytes,
            vec![
                32, 84, 104, 105, 115, 84, 104, 97, 116, // ThisThat
                24, 68, 111, 84, 104, 97, 116, // DoThat
                255, 255, 255, 255, // p1
                20, 104, 101, 108, 108, 111, // p2
                0    // p3
            ]
        );
    }

    #[test]
    fn test_io_module_decode_reply() {
        let bytes = vec![
            32, 84, 104, 105, 115, 84, 104, 97, 116, // ThisThat
            24, 68, 111, 84, 104, 97, 116, // DoThat
            0,   // Ok
            16, 65, 65, 65, 65, // len + "AAAA"
            255, 255, 255, 255, // u32::MAX
        ];

        let reply: Result<(String, u32), (String,)> =
            this_that_io::DoThat::decode_reply(&bytes).unwrap();

        assert_eq!(reply, Ok(("AAAA".to_string(), u32::MAX)));
    }
}
