pub use errors::*;
use handlebars::{Handlebars, handlebars_helper};
use meta::ExpandedProgramMeta;
pub use program::*;
use scale_info::{Field, PortableType, Variant, form::PortableForm};
use serde::Serialize;
use std::{fs, io::Write, path::Path};

mod errors;
mod meta;
mod type_names;

/*
todo [sab] tests:
1. test Invariant for the types: all types there are known (whether built-in or user-defined)
2. test proper indentations when some sections are missing
3. test function no results

*/
// todo [sab] Maps?
// todo [sab] generics?
// todo [sab] adjust Result to throws
// todo [sab] @nonzero for the NonZero rust types.
// todo [sab] unit structs (no fields or empty fields)
// todo [sab] take all the names from type_name
// todo [sab] add service ctors into program section of idl
// todo [sab] add global annotations
// todo [sab] change fields to args
// todo [sab] which sections can be absent -> adjust template with ifs and add proper indentations
// todo [sab] test same type used in multiple services

const IDLV2_TEMPLATE: &str = include_str!("../hbs/idlv2.hbs");
const SERVICE_TEMPLATE: &str = include_str!("../hbs/service.hbs");

pub mod program {
    use super::*;
    use sails_idl_meta::ProgramMeta;

    pub fn generate_idl<P: ProgramMeta>(idl_writer: impl Write) -> Result<()> {
        render_idl(
            &ExpandedProgramMeta::new(Some(P::constructors()), P::services())?,
            idl_writer,
        )
    }

    pub fn generate_idl_to_file<P: ProgramMeta>(path: impl AsRef<Path>) -> Result<()> {
        let mut idl_new_content = Vec::new();
        generate_idl::<P>(&mut idl_new_content)?;
        if let Ok(idl_old_content) = fs::read(&path)
            && idl_new_content == idl_old_content
        {
            return Ok(());
        }
        if let Some(dir_path) = path.as_ref().parent() {
            fs::create_dir_all(dir_path)?;
        }
        Ok(fs::write(&path, idl_new_content)?)
    }
}

pub mod service {
    use super::*;
    use sails_idl_meta::{AnyServiceMeta, ServiceMeta};

    pub fn generate_idl<S: ServiceMeta>(idl_writer: impl Write) -> Result<()> {
        render_idl(
            &ExpandedProgramMeta::new(None, vec![("", AnyServiceMeta::new::<S>())].into_iter())?,
            idl_writer,
        )
    }

    pub fn generate_idl_to_file<S: ServiceMeta>(path: impl AsRef<Path>) -> Result<()> {
        let mut idl_new_content = Vec::new();
        generate_idl::<S>(&mut idl_new_content)?;
        if let Ok(idl_old_content) = fs::read(&path)
            && idl_new_content == idl_old_content
        {
            return Ok(());
        }
        Ok(fs::write(&path, idl_new_content)?)
    }
}

fn render_idl(program_meta: &ExpandedProgramMeta, idl_writer: impl Write) -> Result<()> {
    let program_idl_data = ProgramIdlData {
        type_names: program_meta.type_names()?.collect(),
        // Only Program types, not builtins, not commands/queries/events, not native Rust types
        types: program_meta.types().collect(),
        ctors: program_meta.ctors().collect(),
        services: program_meta
            .services()
            .map(|s| ServiceIdlData {
                name: s.name(),
                commands: s.commands().collect(),
                queries: s.queries().collect(),
                events: s.events().collect(),
            })
            .collect(),
    };

    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string("idlv2", IDLV2_TEMPLATE)
        .map_err(Box::new)?;
    handlebars
        .register_template_string("service", SERVICE_TEMPLATE)
        .map_err(Box::new)?;
    handlebars.register_helper("deref", Box::new(deref));

    handlebars
        .render_to_write("idl", &program_idl_data, idl_writer)
        .map_err(Box::new)?;

    Ok(())
}

type CtorIdlData<'a> = (&'a str, &'a Vec<Field<PortableForm>>, &'a Vec<String>);
type FuncIdlData<'a> = (&'a str, &'a Vec<Field<PortableForm>>, u32, &'a Vec<String>);

#[derive(Serialize)]
struct ProgramIdlData<'a> {
    type_names: Vec<String>,
    types: Vec<&'a PortableType>,
    ctors: Vec<CtorIdlData<'a>>,
    services: Vec<ServiceIdlData<'a>>,
}

#[derive(Serialize)]
struct Idl2Data {
    #[serde(rename = "program")]
    program_section: ProgramIdlSection,
    services: Vec<ServiceSection>,
}

#[derive(Serialize)]
struct ProgramIdlSection {
    name: String,
    type_names: Vec<String>,
    ctors: Vec<FunctionIdl2Data>,
}

#[derive(Serialize)]
struct FunctionIdl2Data {
    name: String,
    args: Vec<FuncArgIdl2>,
    // () return value is no-op
    #[serde(skip_serializing_if = "Option::is_none")]
    result_ty: Option<u32>,
    docs: Vec<String>,
}

#[derive(Serialize)]
struct FuncArgIdl2 {
    name: String,
    #[serde(rename = "type_idx")]
    ty: u32,
}

#[derive(Serialize)]
struct ServiceSection {
    name: String,
    type_names: Vec<String>,
    extends: Vec<String>,
    events: Vec<Variant<PortableForm>>,
    types: Vec<PortableType>,
    functions: FunctionsSection,
}

#[derive(Serialize)]
struct FunctionsSection {
    commands: Vec<FunctionIdl2Data>,
    queries: Vec<FunctionIdl2Data>,
}

#[derive(Serialize)]
struct ServiceIdlData<'a> {
    name: &'a str,
    commands: Vec<FuncIdlData<'a>>,
    queries: Vec<FuncIdlData<'a>>,
    events: Vec<&'a Variant<PortableForm>>,
}

handlebars_helper!(deref: |v: String| { v });

#[cfg(test)]
mod tests {
    use crate::meta::ExpandedProgramMeta2;

    use super::*;

    #[test]
    fn test_real_hbs() {
        let data = serde_json::json!({
            "type_names": [
                "actor_id",
                "[u8, 32]",
                "u8",
                "code_id",
                "message_id",
                "h160",
                "[u8, 20]",
                "h256",
                "u256",
                "[u64, 4]",
                "u64",
                "nat256",
                "ConstructorsMeta",
                "DefaultParams",
                "NewParams",
                "opt u32",
                "u32",
                "opt struct { i32, i32 }",
                "struct { i32, i32 }",
                "i32",
                "PingServiceMetaCommandsMeta",
                "PingParams",
                "str",
                "result (str, str)",
                "PingServiceMetaQueriesMeta",
                "PingServiceMetaNoEvents",
                "CounterServiceMetaCommandsMeta",
                "CounterServiceMetaAddParams",
                "SubParams",
                "CounterServiceMetaQueriesMeta",
                "ValueParams",
                "CounterEvents",
                "DogServiceMetaCommandsMeta",
                "DogServiceMetaMakeSoundParams",
                "MammalServiceMetaCommandsMeta",
                "MammalServiceMetaMakeSoundParams",
                "WalkerServiceMetaCommandsMeta",
                "WalkParams",
                "null",
                "DogServiceMetaQueriesMeta",
                "MammalServiceMetaQueriesMeta",
                "AvgWeightParams",
                "WalkerServiceMetaQueriesMeta",
                "PositionParams",
                "DogEvents",
                "MammalServiceMetaNoEvents",
                "WalkerEvents",
                "ReferenceServiceMetaCommandsMeta",
                "ReferenceServiceMetaAddParams",
                "AddByteParams",
                "vec u8",
                "GuessNumParams",
                "result (str, str)",
                "IncrParams",
                "ReferenceCount",
                "SetNumParams",
                "result (null, str)",
                "ReferenceServiceMetaQueriesMeta",
                "BakedParams",
                "LastByteParams",
                "opt u8",
                "MessageParams",
                "opt str",
                "ReferenceServiceMetaNoEvents",
                "MyServiceMetaCommandsMeta",
                "DoThatParams",
                "DoThatParam",
                "nat32",
                "ManyVariants",
                "opt u256",
                "opt u16",
                "u16",
                "struct { u32 }",
                "result (struct { actor_id, nat32, ManyVariantsReply }, struct { str })",
                "struct { actor_id, nat32, ManyVariantsReply }",
                "ManyVariantsReply",
                "struct { str }",
                "DoThisParams",
                "struct { opt h160, nat8 }",
                "opt h160",
                "nat8",
                "TupleStruct",
                "bool",
                "struct { str, u32 }",
                "NoopParams",
                "MyServiceMetaQueriesMeta",
                "ThatParams",
                "ThisParams",
                "MyServiceMetaNoEvents",
                "FeeServiceMetaCommandsMeta",
                "DoSomethingAndTakeFeeParams",
                "FeeServiceMetaQueriesMeta",
                "FeeEvents",
                "u128"
            ],
            "program": {
                "name": "Demo",
                "ctors": [
                    {
                        "name": "Default",
                        "args": [],
                        "docs": [
                            "Program constructor (called once at the very beginning of the program lifetime)"
                        ]
                    },
                    {
                        "name": "New",
                        "args": [
                            {
                                "name": "counter",
                                "type": 15,
                                "typeName": "Option<u32>"
                            },
                            {
                                "name": "dog_position",
                                "type": 17,
                                "typeName": "Option<(i32, i32)>"
                            }
                        ],
                        "docs": [
                            "Another program constructor (called once at the very beginning of the program lifetime)"
                        ]
                    }
                ]
            },
            "services": [
                {
                    "name": "ThisThat",
                    "extends": ["Mammal", "Pet"],
                    "events": [
                        {
                            "name": "Barked",
                            "index": 0
                        },
                        {
                            "name": "Walked",
                            "fields": [
                                {
                                    "name": "from",
                                    "type": 18,
                                    "typeName": "(i32, i32)"
                                },
                                {
                                    "name": "to",
                                    "type": 18,
                                    "typeName": "(i32, i32)"
                                }
                            ],
                            "index": 0
                        },
                        {
                            "name": "Meowed",
                            "index": 0,
                            "fields": [
                                {
                                    "type": 16,
                                    "typeName": "u32"
                                },
                                {
                                    "type": 16,
                                    "typeName": "u32"
                                }
                            ]
                        }
                    ],
                    "functions": {
                        "commands": [
                            {
                                "name": "DoThat",
                                "args": [
                                    {
                                        "name": "param",
                                        "type": 66,
                                        "typeName": "DoThatParam"
                                    }
                                ],
                                "result": "Result<(ActorId, NonZeroU32, ManyVariantsReply), (String,)>",
                                "docs": []
                            },
                            {
                                "name": "DoThis",
                                "args": [
                                    {
                                        "name": "p1",
                                        "type": 16,
                                        "typeName": "u32"
                                    },
                                    {
                                        "name": "p2",
                                        "type": 22,
                                        "typeName": "String"
                                    },
                                    {
                                        "name": "p3",
                                        "type": 78,
                                        "typeName": "(Option<H160>, NonZeroU8)"
                                    },
                                    {
                                        "name": "p4",
                                        "type": 81,
                                        "typeName": "TupleStruct"
                                    }
                                ],
                                "result": "(String, u32)",
                                "docs": []
                            },
                            {
                                "name": "Noop",
                                // "args": [],
                                "result": "",
                                "docs": []
                            }
                        ],
                        "queries": [
                            {
                                "name": "That",
                                "args": [],
                                "result": "Result<String, String>",
                                "docs": []
                            },
                            {
                                "name": "This",
                                "result": "u32",
                                "docs": []
                            }
                        ]
                    },
                    "types": [
                        {
                            "id": 54,
                            "type": {
                                "path": [
                                    "demo",
                                    "references",
                                    "ReferenceCount"
                                ],
                                "def": {
                                    "composite": {
                                        "fields": [
                                            {
                                                "type": 16,
                                                "typeName": "u32"
                                            }
                                        ]
                                    }
                                }
                            },
                        },
                        {
                            "id": 66,
                            "type": {
                                "path": [
                                    "demo",
                                    "this_that",
                                    "DoThatParam"
                                ],
                                "def": {
                                    "composite": {
                                        "fields": [
                                            {
                                                "name": "p1",
                                                "type": 67,
                                                "typeName": "NonZeroU32"
                                            },
                                            {
                                                "name": "p2",
                                                "type": 0,
                                                "typeName": "ActorId"
                                            },
                                            {
                                                "name": "p3",
                                                "type": 68,
                                                "typeName": "ManyVariants"
                                            }
                                        ]
                                    }
                                }
                            }
                        },
                        {
                            "id": 68,
                            "type": {
                                "path": [
                                    "demo",
                                    "this_that",
                                    "ManyVariants"
                                ],
                                "def": {
                                    "variant": {
                                        "variants": [
                                            {
                                                "name": "One",
                                                "index": 0
                                            },
                                            {
                                                "name": "Two",
                                                "fields": [
                                                    {
                                                        "type": 16,
                                                        "typeName": "u32"
                                                    }
                                                ],
                                                "index": 1
                                            },
                                            {
                                                "name": "Three",
                                                "fields": [
                                                    {
                                                        "type": 69,
                                                        "typeName": "Option<U256>"
                                                    }
                                                ],
                                                "index": 2
                                            },
                                            {
                                                "name": "Four",
                                                "fields": [
                                                    {
                                                        "name": "a",
                                                        "type": 16,
                                                        "typeName": "u32"
                                                    },
                                                    {
                                                        "name": "b",
                                                        "type": 70,
                                                        "typeName": "Option<u16>"
                                                    }
                                                ],
                                                "index": 3
                                            },
                                            {
                                                "name": "Five",
                                                "fields": [
                                                    {
                                                        "type": 22,
                                                        "typeName": "String"
                                                    },
                                                    {
                                                        "type": 7,
                                                        "typeName": "H256"
                                                    }
                                                ],
                                                "index": 4
                                            },
                                            {
                                                "name": "Six",
                                                "fields": [
                                                    {
                                                        "type": 72,
                                                        "typeName": "(u32,)"
                                                    }
                                                ],
                                                "index": 5
                                            }
                                        ]
                                    }
                                }
                            }
                        },
                        {
                            "id": 75,
                            "type": {
                                "path": [
                                    "demo",
                                    "this_that",
                                    "ManyVariantsReply"
                                ],
                                "def": {
                                    "variant": {
                                        "variants": [
                                            {
                                                "name": "One",
                                                "index": 0
                                            },
                                            {
                                                "name": "Two",
                                                "index": 1
                                            },
                                            {
                                                "name": "Three",
                                                "index": 2
                                            },
                                            {
                                                "name": "Four",
                                                "index": 3
                                            },
                                            {
                                                "name": "Five",
                                                "index": 4
                                            },
                                            {
                                                "name": "Six",
                                                "index": 5
                                            }
                                        ]
                                    }
                                }
                            }
                        },
                        {
                            "id": 81,
                            "type": {
                                "path": [
                                    "demo",
                                    "this_that",
                                    "TupleStruct"
                                ],
                                "def": {
                                    "composite": {
                                        "fields": [
                                            {
                                                "type": 82,
                                                "typeName": "bool"
                                            }
                                        ]
                                    }
                                }
                            }
                        }
                    ],
                }
            ]
        });

        let mut source: Vec<u8> = Vec::new();
        let mut hbs = Handlebars::new();
        let _ = hbs.register_template_string("idlv2", IDLV2_TEMPLATE);
        let _ = hbs.register_template_string("service", SERVICE_TEMPLATE);

        hbs.register_helper("deref", Box::new(deref));
        hbs.render_to_write("idlv2", &data, &mut source).unwrap();

        println!("{}", String::from_utf8_lossy(&source));
    }

    #[test]
    fn test_new_json() {
        use sails_idl_meta::ProgramMeta;
        use demo::DemoProgram;
        let mut source: Vec<u8> = Vec::new();

        let data = ExpandedProgramMeta2::new(
            "Demo".to_string(),
            DemoProgram::constructors(),
            DemoProgram::services(),
        ).unwrap();

        let json = serde_json::to_string_pretty(&data).unwrap();
        println!("{}", json);

        let mut hbs = Handlebars::new();
        let _ = hbs.register_template_string("idlv2", IDLV2_TEMPLATE);
        let _ = hbs.register_template_string("service", SERVICE_TEMPLATE);
        hbs.register_helper("deref", Box::new(deref));

        hbs.render_to_write("idlv2", &data, &mut source).unwrap();
        println!("{}", String::from_utf8_lossy(&source));
    }
}