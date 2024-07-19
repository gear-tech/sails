#[test]
fn full() {
    const IDL: &str = r#"
        type ThisThatSvcAppTupleStruct = struct {
            bool,
        };

        type ThisThatSvcAppDoThatParam = struct {
            p1: u32,
            p2: str,
            p3: ThisThatSvcAppManyVariants,
        };

        type ThisThatSvcAppManyVariants = enum {
            One,
            Two: u32,
            Three: opt u32,
            Four: struct { a: u32, b: opt u16 },
            Five: struct { str, u32 },
            Six: struct { u32 },
        };

        type T = enum { One };

        constructor {
            New : (a: u32);
        };

        service {
            DoThis : (p1: u32, p2: str, p3: struct { opt str, u8 }, p4: ThisThatSvcAppTupleStruct) -> struct { str, u32 };
            DoThat : (param: ThisThatSvcAppDoThatParam) -> result (struct { str, u32 }, struct { str });
            query This : (v1: vec u16) -> u32;
            query That : (v1: null) -> result (str, str);
        };
        "#;

    insta::assert_snapshot!(gen(IDL, "Service"));
}

#[test]
fn test_basic_works() {
    let idl = r"
        type MyParam = struct {
            f1: u32,
            f2: vec str,
            f3: opt struct { u8, u32 },
        };

        type MyParam2 = enum {
            Variant1,
            Variant2: u32,
            Variant3: struct { u32 },
            Variant4: struct { u8, u32 },
            Variant5: struct { f1: str, f2: vec u8 },
        };

        service {
            DoThis: (p1: u32, p2: MyParam) -> u16;
            DoThat: (p1: struct { u8, u32 }) -> u8;
        };
    ";

    insta::assert_snapshot!(gen(idl, "Basic"));
}

#[test]
fn test_multiple_services() {
    let idl = r"
        service {
            DoThis: (p1: u32, p2: MyParam) -> u16;
            DoThat: (p1: struct { u8, u32 }) -> u8;
        };

        service Named {
            query That: (p1: u32) -> str;
        };
    ";

    insta::assert_snapshot!(gen(idl, "Multiple"));
}

#[test]
fn test_rmrk_works() {
    let idl = include_str!("../../../examples/rmrk/catalog/wasm/rmrk-catalog.idl");

    insta::assert_snapshot!(gen(idl, "RmrkCatalog"));
}

#[test]
fn test_nonzero_works() {
    let idl = r"
            type MyParam = struct {
                f1: nat256,
                f2: vec nat8,
                f3: opt struct { nat64, nat256 },
            };

            service {
                DoThis: (p1: nat256, p2: MyParam) -> nat64;
            };
        ";

    insta::assert_snapshot!(gen(idl, "NonZeroParams"));
}

#[test]
fn test_events_works() {
    let idl = r"
            type MyParam = struct {
                f1: nat256,
                f2: vec nat8,
                f3: opt struct { nat64, nat256 },
            };

            service {
                DoThis: (p1: nat256, p2: MyParam) -> nat64;

                events {
                    One: u64;
                    Two: struct { id: u8, reference: u64 };
                    Three: MyParam;
                    Reset;
                }
            };
        ";

    insta::assert_snapshot!(gen(idl, "ServiceWithEvents"));
}

fn gen(program: &str, service_name: &str) -> String {
    let program = sails_idl_parser::ast::parse_idl(program).expect("parse IDL");

    sails_client_gen::generate(program, service_name, Some("mockall".to_owned()))
        .expect("generate client")
}
