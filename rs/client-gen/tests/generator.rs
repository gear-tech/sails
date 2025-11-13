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
    let idl = r#"
        service Basic {
            functions {
                DoThis(p1: u32, p2: MyParam) -> u16;
                DoThat(p1: (u8, u32)) -> u8;
            }
            types {
                struct MyParam {
                    f1: u32,
                    f2: [string],
                    f3: Option<(u8, u32)>,
                }

                enum MyParam2 {
                    Variant1,
                    Variant2(u32),
                    Variant3(u32),
                    Variant4(u8, u32),
                    Variant5 { f1: string, f2: [u8] },
                }
            }
        }
    "#;

    insta::assert_snapshot!(gen_client(idl, "Basic"));
}

#[test]
fn test_multiple_services() {
    let idl = r#"
        service Multiple { // Anonymous service becomes named "Multiple" for the test
            functions {
                DoThis(p1: u32, p2: MyParam) -> u16;
                DoThat(p1: (u8, u32)) -> u8;
            }
            types {
                // MyParam is not defined in this IDL, it's assumed to be external or defined elsewhere.
                // For this test, I'll define a dummy MyParam to make it self-contained.
                struct MyParam {
                    value: u32,
                }
            }
        }

        service Named {
            functions {
                @query
                That(p1: u32) -> string;
            }
        }
    "#;

    insta::assert_snapshot!(gen_client(idl, "Multiple"));
}

#[test]
fn test_rmrk_works() {
    const IDL: &str = r#"
        program RmrkCatalog {
            constructors {
                New();
            }
            services {
                RmrkCatalogService,
            }
            types {
                enum Error {
                    PartIdCantBeZero,
                    BadConfig,
                    PartAlreadyExists,
                    ZeroLengthPassed,
                    PartDoesNotExist,
                    WrongPartFormat,
                    NotAllowedToCall,
                }

                enum Part {
                    Fixed(FixedPart),
                    Slot(SlotPart),
                }

                struct FixedPart {
                    /// An optional zIndex of base part layer.
                    /// specifies the stack order of an element.
                    /// An element with greater stack order is always in front of an element with a lower stack order.
                    z: Option<u32>,
                    /// The metadata URI of the part.
                    metadata_uri: string,
                }

                struct SlotPart {
                    /// Array of whitelisted collections that can be equipped in the given slot. Used with slot parts only.
                    equippable: Vec<ActorId>,
                    /// An optional zIndex of base part layer.
                    /// specifies the stack order of an element.
                    z: Option<u32>,
                    /// The metadata URI of the part.
                    metadata_uri: string,
                }
            }
        }

        service RmrkCatalogService {
            functions {
                AddEquippables(part_id: u32, collection_ids: Vec<ActorId>) -> Result<(u32, Vec<ActorId>), Error>;
                AddParts(parts: BTreeMap<u32, Part>) -> Result<BTreeMap<u32, Part>, Error>;
                RemoveEquippable(part_id: u32, collection_id: ActorId) -> Result<(u32, ActorId), Error>;
                RemoveParts(part_ids: Vec<u32>) -> Result<Vec<u32>, Error>;
                ResetEquippables(part_id: u32) -> Result<(), Error>;
                SetEquippablesToAll(part_id: u32) -> Result<(), Error>;
                @query
                Equippable(part_id: u32, collection_id: ActorId) -> Result<bool, Error>;
                @query
                Part(part_id: u32) -> Option<Part>;
            }
        }
    "#;

    insta::assert_snapshot!(gen_client(IDL, "RmrkCatalog"));
}

#[test]
fn test_nonzero_works() {
    let idl = r#"
        service NonZeroParams {
            functions {
                DoThis(p1: U256, p2: MyParam) -> U64;
            }
            types {
                struct MyParam {
                    f1: U256,
                    f2: [u8],
                    f3: Option<(U64, U256)>,
                }
            }
        }
    "#;

    insta::assert_snapshot!(gen_client(idl, "NonZeroParams"));
}

#[test]
fn test_events_works() {
    let idl = r#"
        service ServiceWithEvents {
            functions {
                DoThis(p1: U256, p2: MyParam) -> U64;
            }

            events {
                One(u64),
                Two { id: u8, reference: u64 },
                Three(MyParam),
                Reset,
            }

            types {
                struct MyParam {
                    f1: U256,
                    f2: [u8],
                    f3: Option<(U64, U256)>,
                }
            }
        }
    "#;

    insta::assert_snapshot!(gen_client(idl, "ServiceWithEvents"));
}

#[test]
fn full_with_sails_path() {
    const IDL: &str = r#"
        program Service { // The anonymous service is now part of a program
            constructors {
                /// New constructor
                New(a: u32);
                /// CreateWithData constructor
                CreateWithData(a: u32, b: string, c: ThisThatSvcAppManyVariants);
            }
            services {
                ThisThatService,
                CounterService,
            }
            types {
                struct ThisThatSvcAppTupleStruct(bool);

                struct ThisThatSvcAppDoThatParam {
                    p1: u32,
                    p2: string,
                    p3: ThisThatSvcAppManyVariants,
                }

                enum ThisThatSvcAppManyVariants {
                    One,
                    Two(u32),
                    Three(Option<u32>),
                    Four { a: u32, b: Option<u16> },
                    Five(string, u32),
                    Six(u32),
                }

                enum T { One } // This was a type T = enum { One };
            }
        }

        service ThisThatService {
            functions {
                DoThis(p1: u32, p2: string, p3: Option<(string, u8)>, p4: ThisThatSvcAppTupleStruct) -> (string, u32);
                DoThat(param: ThisThatSvcAppDoThatParam) -> Result<(string, u32), string>;
                @query
                This(v1: Vec<u16>) -> u32;
                @query
                That(v1: ()) -> Result<string, string>;
            }
        }

        service CounterService {
            functions {
                /// Add a value to the counter
                Add(value: u32) -> u32;
                /// Substract a value from the counter
                Sub(value: u32) -> u32;
                /// Get the current value
                @query
                Value() -> u32;
            }

            events {
                /// Emitted when a new value is added to the counter
                Added(u32),
                /// Emitted when a value is subtracted from the counter
                Subtracted(u32),
            }
        }
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
        service Service { // Anonymous service becomes named "Service" for the test
            functions {
                DoThis(p1: u32, p2: MyParam) -> u16;
                DoThat(p1: (u8, u32)) -> u8;
            }
            types {
                struct MyParam {
                    f1: u32,
                    f2: [string],
                    f3: Option<(u8, u32)>,
                }

                enum MyParam2 {
                    Variant1,
                    Variant2(u32),
                    Variant3(u32),
                    Variant4(u8, u32),
                    Variant5 { f1: string, f2: [u8] },
                }
            }
        }
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
