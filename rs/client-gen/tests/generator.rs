use sails_client_gen::ClientGenerator;

#[test]
fn full() {
    const IDL: &str = r#"
        // Comments are supported but ignored by idl-parser
        
        /// ThisThatSvcAppTupleStruct docs
        type ThisThatSvcAppTupleStruct = struct {
            /// field `bool`
            bool,
        };

        /// ThisThatSvcAppDoThatParam docs
        type ThisThatSvcAppDoThatParam = struct {
            /// field `query`
            query: u32,
            /// field `result`
            result: str,
            /// field `p3`
            p3: ThisThatSvcAppManyVariants,
        };

        /// ThisThatSvcAppManyVariants docs
        type ThisThatSvcAppManyVariants = enum {
            /// variant `One` 
            One,
            /// variant `Two`
            Two: u32,
            Three: opt u32,
            Four: struct { a: u32, b: opt u16 },
            Five: struct { str, u32 },
            Six: struct { u32 },
        };

        type T = enum { One };

        constructor {
            /// New constructor
            New : (a: u32);
        };

        service {
            /// Some description
            DoThis : (p1: u32, p2: str, p3: struct { opt str, u8 }, p4: ThisThatSvcAppTupleStruct) -> struct { str, u32 };
            /// Some multiline description
            /// Second line
            /// Third line
            DoThat : (param: ThisThatSvcAppDoThatParam) -> result (struct { str, u32 }, struct { str });
            /// This is a query
            query This : (v1: vec u16) -> u32;
            /// This is a second query
            /// This is a second line
            query That : (v1: null) -> result (str, str);

            events {
                /// `This` Done
                ThisDone: u32;
                /// `That` Done too
                ThatDone: struct {
                    /// This is `p1` field
                    p1: str
                };
            }
        };
        "#;

    insta::assert_snapshot!(gen_client(IDL, "Service"));
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

    insta::assert_snapshot!(gen_client(idl, "Basic"));
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

    insta::assert_snapshot!(gen_client(idl, "Multiple"));
}

#[test]
fn test_rmrk_works() {
    let idl = include_str!("../../../examples/rmrk/catalog/wasm/rmrk-catalog.idl");

    insta::assert_snapshot!(gen_client(idl, "RmrkCatalog"));
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

    insta::assert_snapshot!(gen_client(idl, "NonZeroParams"));
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

    insta::assert_snapshot!(gen_client(idl, "ServiceWithEvents"));
}

#[test]
fn full_with_sails_path() {
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

    let code = ClientGenerator::from_idl(IDL)
        .with_sails_crate("my_crate::sails")
        .generate("Service")
        .expect("generate client");
    insta::assert_snapshot!(code);
}

#[test]
fn test_external_types() {
    const IDL: &str = r#"
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
        "#;

    let code = ClientGenerator::from_idl(IDL)
        .with_sails_crate("my_crate::sails")
        .with_external_type("MyParam", "my_crate::MyParam")
        .with_no_derive_traits()
        .generate("Service")
        .expect("generate client");
    insta::assert_snapshot!(code);
}

fn gen_client(program: &str, service_name: &str) -> String {
    ClientGenerator::from_idl(program)
        .with_mocks("with_mocks")
        .generate(service_name)
        .expect("generate client")
}
