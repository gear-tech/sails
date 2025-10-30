// =============================
// src/parser.rs — AST + builder from Pairs
// =============================
use anyhow::{Result, bail};
use pest::Parser;
use pest::iterators::{Pair, Pairs};

mod model;
pub use model::*;

#[derive(pest_derive::Parser)]
// #[grammar_skip(WHITESPACE, COMMENT, NEWLINE)]
#[grammar = "idl.pest"]
pub struct IdlParser;

// ----------------------------- Public API ------------------------------------
impl IdlDoc {
    pub fn parse(src: &str) -> Result<Self> {
        let mut pairs = IdlParser::parse(Rule::Top, src)?;
        build_idl(pairs.next().unwrap())
    }
}

// ------------------------------- Builders ------------------------------------
fn build_idl(top: Pair<Rule>) -> Result<IdlDoc> {
    let mut globals = Vec::new();
    let mut services = Vec::new();
    let mut program = None;
    for p in top.into_inner() {
        match p.as_rule() {
            Rule::GlobalAnn => {
                let (k, v) = parse_ann(p);
                globals.push((k, v));
            }
            // Rule::ServiceDecl => items.push(TopItem::Service(parse_service(p)?)),
            Rule::ProgramDecl => program = Some(parse_program(p)?),
            _ => {}
        }
    }
    Ok(IdlDoc {
        globals,
        services,
        program,
    })
}

fn parse_ann(p: Pair<Rule>) -> (String, Option<String>) {
    let mut key = None;
    let mut val = None;
    for i in p.into_inner() {
        match i.as_rule() {
            Rule::Ident => key = Some(i.as_str().trim().to_string()),
            Rule::Value => val = Some(strip_line(i.as_str())),
            _ => {}
        }
    }
    (key.unwrap(), val)
}

// fn parse_service(p: Pair<Rule>) -> Result<Service> {
//     let mut it = p.into_inner();
//     let (docs, anns) = collect_docs_anns_prefix(&mut it);
//     let name = expect_rule(&mut it, Rule::ident)?.as_str().to_string();
//     let mut extends = Vec::new();
//     let mut events = Vec::new();
//     let mut functions = Vec::new();
//     let mut types_src = Vec::new();
//     for item in it {
//         match item.as_rule() {
//             Rule::ExtendsBlock => {
//                 for path in item.into_inner().filter(|x| x.as_rule() == Rule::Path) {
//                     extends.push(path.as_str().to_string());
//                 }
//             }
//             Rule::EventsBlock => {
//                 for e in item.into_inner().filter(|x| x.as_rule() == Rule::EventDecl) {
//                     events.push(parse_event(e)?);
//                 }
//             }
//             Rule::FunctionsBlock => {
//                 for f in item.into_inner().filter(|x| x.as_rule() == Rule::FuncDecl) {
//                     functions.push(parse_func(f)?);
//                 }
//             }
//             Rule::TypesBlock => {
//                 types_src.push(item.as_str().to_string());
//             }
//             _ => {}
//         }
//     }
//     Ok(Service {
//         name,
//         extends,
//         events,
//         functions,
//         types_src,
//         docs,
//         annotations: anns,
//     })
// }

// fn parse_event(p: Pair<Rule>) -> Result<EnumVariant> {
//     // EventDecl wraps Variant
//     let v = p.into_inner().next().unwrap();
//     parse_variant(v)
// }

// fn parse_variant(p: Pair<Rule>) -> Result<EnumVariant> {
//     let mut it = p.into_inner().peekable();
//     let (docs, anns) = collect_docs_anns_prefix(&mut it);
//     let name = expect_rule_peek(&mut it, Rule::ident)?.as_str().to_string();
//     it.next();
//     let mut fields = Vec::new();
//     let mut tuple = Vec::new();
//     if let Some(next) = it.peek() {
//         match next.as_rule() {
//             Rule::VariantStruct => {
//                 let f = parse_fields(next.clone());
//                 fields = f;
//                 it.next();
//             }
//             Rule::VariantTuple => {
//                 let t = parse_typelist(next.clone());
//                 tuple = t;
//                 it.next();
//             }
//             _ => {}
//         }
//     }
//     Ok(EnumVariant {
//         name,
//         fields,
//         tuple,
//         docs,
//         annotations: anns,
//     })
// }

// fn parse_fields(p: Pair<Rule>) -> Vec<(String, TypeDecl)> {
//     let mut v = Vec::new();
//     for f in p.into_inner().filter(|x| x.as_rule() == Rule::Field) {
//         let mut it = f.into_inner();
//         // consume optional Docs/Anns if present
//         let _ = collect_docs_anns_prefix(&mut it);
//         let name = it.next().unwrap().as_str().to_string();
//         let ty = TypeDecl {
//             text: it.next().unwrap().as_str().to_string(),
//         };
//         v.push((name, ty));
//     }
//     v
// }

// fn parse_typelist(p: Pair<Rule>) -> Vec<TypeDecl> {
//     p.into_inner()
//         .filter(|x| x.as_rule() == Rule::Type)
//         .map(|t| TypeDecl {
//             text: t.as_str().to_string(),
//         })
//         .collect()
// }

// fn parse_func(p: Pair<Rule>) -> Result<ServiceFunc> {
//     let mut it = p.into_inner();
//     let (docs, anns) = collect_docs_anns_prefix(&mut it);
//     let name = expect_rule(&mut it, Rule::ident)?.as_str().to_string();
//     let mut params = Vec::new();
//     let mut output = None;
//     let mut throws = None;
//     let mut is_query = false;
//     for part in it {
//         match part.as_rule() {
//             Rule::Params => {
//                 for prm in part.into_inner().filter(|x| x.as_rule() == Rule::Param) {
//                     let mut pit = prm.into_inner();
//                     let p_name = pit.next().unwrap().as_str().to_string();
//                     let p_ty = TypeDecl {
//                         text: pit.next().unwrap().as_str().to_string(),
//                     };
//                     params.push(FuncParam {
//                         name: p_name,
//                         type_decl: p_ty,
//                     });
//                 }
//             }
//             Rule::Ret => {
//                 output = Some(TypeDecl {
//                     text: part.into_inner().next().unwrap().as_str().to_string(),
//                 })
//             }
//             Rule::Throws => {
//                 throws = Some(TypeDecl {
//                     text: part.into_inner().next().unwrap().as_str().to_string(),
//                 })
//             }
//             Rule::Anns => {
//                 // capture @query, etc.
//                 for a in part.into_inner() {
//                     let a = parse_local_ann(a);
//                     if a.key == "query" {
//                         is_query = true;
//                     }
//                 }
//             }
//             _ => {}
//         }
//     }
//     Ok(ServiceFunc {
//         name,
//         params,
//         output,
//         throws,
//         is_query,
//         docs,
//         annotations: anns,
//     })
// }

fn parse_ctor_func(p: Pair<Rule>) -> Result<CtorFunc> {
    let mut it = p.into_inner();
    let (docs, annotations) = parse_local_anns_and_docs(&mut it);
    let name = expect_rule(&mut it, Rule::Ident)?.as_str().to_string();
    let mut params = Vec::new();
    for part in it {
        match part.as_rule() {
            Rule::Params => {
                for prm in part.into_inner().filter(|x| x.as_rule() == Rule::Param) {
                    // let mut pit = prm.into_inner();
                    // let p_name = pit.next().unwrap().as_str().to_string();
                    // let p_ty = TypeDecl {
                    //     text: pit.next().unwrap().as_str().to_string(),
                    // };
                    // params.push(FuncParam {
                    //     name: p_name,
                    //     type_decl: p_ty,
                    // });
                }
            }
            _ => {}
        }
    }
    Ok(CtorFunc {
        name,
        params,
        docs,
        annotations,
    })
}

fn parse_program(p: Pair<Rule>) -> Result<ProgramUnit> {
    let mut it = p.into_inner();
    // println!("{:#?}", it);
    let (docs, annotations) = parse_local_anns_and_docs(&mut it);
    println!("{:#?}", it);
    let name = expect_rule(&mut it, Rule::Ident)?.as_str().to_string();
    let mut ctors = Vec::new();
    let mut services = Vec::new();
    let mut types = Vec::new();
    for item in it {
        match item.as_rule() {
            Rule::ConstructorsBlock => {
                for f in item.into_inner().filter(|x| x.as_rule() == Rule::CtorDecl) {
                    ctors.push(parse_ctor_func(f)?);
                }
            }
            Rule::ServicesBlock => {
                for s in item
                    .into_inner()
                    .filter(|x| x.as_rule() == Rule::ServiceItem)
                {
                    println!("{:?}", s);
                    let mut sit = s.into_inner();
                    let (docs, annotations) = parse_local_anns_and_docs(&mut sit);
                    let mut name = expect_rule(&mut sit, Rule::Ident)?.as_str().to_string();
                    let route = name.clone();
                    if let Some(p) = sit.next() {
                        if p.as_rule() == Rule::Ident {
                            name = p.as_str().to_string();
                        }
                    }
                    services.push(ProgramServiceItem {
                        name,
                        route,
                        docs,
                        annotations,
                    });
                }
            }
            Rule::TypesBlock => {
                // types.push(item.as_str().to_string());
            }
            _ => {}
        }
    }
    Ok(ProgramUnit {
        name,
        ctors,
        services,
        types,
        docs,
        annotations,
    })
}

// ------------------------------ Helpers --------------------------------------
fn strip_line(s: &str) -> String {
    s.trim().to_string()
}

fn parse_local_anns_and_docs(
    pairs: &mut Pairs<Rule>,
) -> (Vec<String>, Vec<(String, Option<String>)>) {
    let mut docs = Vec::new();
    let mut anns = Vec::new();
    // iter over cloned
    for p in pairs.clone() {
        // peek Docs or Anns
        match p.as_rule() {
            Rule::Docs => {
                // pop pair
                let _ = pairs.next();
                for d in p.into_inner() {
                    match d.as_rule() {
                        Rule::Value => docs.push(strip_line(d.as_str())),
                        _ => {}
                    }
                }
            }
            Rule::Anns => {
                // pop pair
                let _ = pairs.next();
                for a in p.into_inner() {
                    anns.push(parse_ann(a));
                }
            }
            _ => break,
        }
    }
    (docs, anns)
}

fn expect_rule<'a>(
    it: &mut impl Iterator<Item = Pair<'a, Rule>>,
    r: Rule,
) -> Result<Pair<'a, Rule>> {
    if let Some(p) = it.next() {
        if p.as_rule() == r {
            return Ok(p);
        }
    }
    bail!("expected {:?}", r)
}
fn expect_rule_peek<'a>(
    it: &mut std::iter::Peekable<impl Iterator<Item = Pair<'a, Rule>>>,
    r: Rule,
) -> Result<Pair<'a, Rule>> {
    if let Some(p) = it.peek() {
        if p.as_rule() == r {
            return Ok(p.clone());
        }
    }
    bail!("expected {:?}", r)
}

// ------------------------------ Tests ----------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_doc_lines() {
        const SRC: &str = r#"
            // Comment
            /// Defines status of some point as colored by somebody or dead for some reason.
            /// Dead point - won't be available for coloring anymore.
            "#;
        let mut pairs = IdlParser::parse(Rule::Docs, SRC).expect("parse idl");
        println!("pairs: {pairs:?}");
        let (docs, _anns) = parse_local_anns_and_docs(&mut pairs);
        println!("docs: {docs:?}");
        assert_eq!(2, docs.len());
        assert_eq!(
            "Defines status of some point as colored by somebody or dead for some reason.",
            docs[0]
        );
        assert_eq!(
            "Dead point - won't be available for coloring anymore.",
            docs[1]
        );
    }

    #[test]
    fn parse_global_annotations() {
        const SRC: &str = r#"
            // Global annotations
            !@sails: 0.1.0
            !@include: ownable.idl
            !@include: git://github.com/some_repo/tippable.idl
            "#;
        let doc = IdlDoc::parse(SRC).expect("parse idl");
        println!("{:?}", doc);
        assert_eq!(3, doc.globals.len());
    }

    #[test]
    fn parse_demo_idl() {
        const SRC: &str = r#"
            !@sails: 0.9.1
            !@author: Test Author
            !@version: 1.0.0

            service PingPong {
                functions {
                    Ping(input: String) -> String throws String;
                }
            }

            service Counter {
                events {
                    /// Emitted when a new value is added to the counter
                    Added(u32),
                    /// Emitted when a value is subtracted from the counter
                    Subtracted(u32),
                }

                functions {
                    /// Add a value to the counter
                    Add(value: u32) -> u32;

                    /// Substract a value from the counter
                    Sub(value: u32) -> u32;

                    /// Get the current value
                    @query
                    Value() -> u32;
                }
            }

            service Dog {
                events {
                    Barked,
                    Walked {
                        from: (i32, i32),
                        to: (i32, i32),
                    },
                }

                functions {
                    MakeSound() -> String;

                    Walk(dx: i32, dy: i32);

                    @query
                    AvgWeight() -> u32;

                    @query
                    Position() -> (i32, i32);
                }
            }

            service References {
                functions {
                    Add(v: u32) -> u32;

                    AddByte(byte: u8) -> [u8];

                    GuessNum(number: u8) -> String throws String;

                    Incr() -> ReferenceCount;

                    SetNum(number: u8) throws String;

                    @query
                    Baked() -> String;

                    @query
                    LastByte() -> Option<u8>;

                    @query
                    Message() -> Option<String>;
                }

                types {
                    struct ReferenceCount(u32);
                }
            }

            service ThisThat {
                functions {
                    DoThat(param: DoThatParam) -> (ActorId, NonZeroU32, ManyVariantsReply) throws (String,);

                    DoThis(p1: u32, p2: String, p3: (Option<H160>, NonZeroU8), p4: TupleStruct) -> (String, u32);

                    Noop();

                    @query
                    That() -> String throws String;

                    @query
                    This() -> u32;
                }

                types {
                    struct DoThatParam {
                        p1: NonZeroU32,
                        p2: ActorId,
                        p3: ManyVariants,
                    }

                    enum ManyVariants {
                        One,
                        Two(u32),
                        Three(Option<u256>),
                        Four {
                            a: u32,
                            b: Option<u16>,
                        },
                        Five(String, H256),
                        Six((u32,)),
                    }

                    enum ManyVariantsReply {
                        One,
                        Two,
                        Three,
                        Four,
                        Five,
                        Six,
                    }

                    struct TupleStruct(bool);
                }
            }

            service ValueFee {
                events {
                    Withheld(u128),
                }

                functions {
                    /// Return flag if fee taken and remain value,
                    /// using special type `CommandReply<T>`
                    DoSomethingAndTakeFee() -> bool;
                }
            }

            program Demo {
                constructors {
                    /// Program constructor (called once at the very beginning of the program lifetime)
                    Default();

                    /// Another program constructor (called once at the very beginning of the program lifetime)
                    New(counter: Option<u32>, dog_position: Option<(i32, i32)>);
                }

                services {
                    PingPong,
                    Counter,
                    Dog,
                    References,
                    ThisThat,
                    ValueFee,
                }
            }
        "#;
        let doc = IdlDoc::parse(SRC).expect("parse idl");
        println!("{:?}", doc);
        assert_eq!(3, doc.globals.len());
    }
}
