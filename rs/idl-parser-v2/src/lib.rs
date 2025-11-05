// Sails IDL v2 — parser using `pest-rs`
use anyhow::{Context, Result, bail};
use pest::Parser;
use pest::iterators::{Pair, Pairs};

mod ast;
pub use ast::*;

#[derive(pest_derive::Parser)]
#[grammar = "idl.pest"]
pub struct IdlParser;

// ----------------------------- Public API ------------------------------------
impl IdlDoc {
    pub fn parse(src: &str) -> Result<Self> {
        let mut pairs = IdlParser::parse(Rule::Top, src)?;
        build_idl(pairs.next().context("expected Top")?)
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
                globals.push(parse_annotation(p)?);
            }
            Rule::ServiceDecl => services.push(parse_service(p)?),
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

fn parse_ident(p: Pair<Rule>) -> Result<String> {
    if p.as_rule() == Rule::Ident {
        return Ok(p.as_str().to_string());
    }
    bail!("expected Ident")
}

fn parse_annotation(p: Pair<Rule>) -> Result<(String, Option<String>)> {
    let mut key = None;
    let mut val = None;
    for i in p.into_inner() {
        match i.as_rule() {
            Rule::Ident => key = Some(i.as_str().trim().to_string()),
            Rule::StrToEol => val = Some(i.as_str().trim().to_string()),
            _ => {}
        }
    }
    let key = key.context("expected Ident")?;
    Ok((key, val))
}

fn map_primitive(s: &str) -> Result<PrimitiveType> {
    Ok(match s {
        // allow both lowercase and PascalCase for some custom types
        "()" => PrimitiveType::Void,
        "bool" | "Bool" => PrimitiveType::Bool,
        "char" | "Char" => PrimitiveType::Char,
        "string" | "String" => PrimitiveType::String,
        "u8" => PrimitiveType::U8,
        "u16" => PrimitiveType::U16,
        "u32" => PrimitiveType::U32,
        "u64" => PrimitiveType::U64,
        "u128" => PrimitiveType::U128,
        "i8" => PrimitiveType::I8,
        "i16" => PrimitiveType::I16,
        "i32" => PrimitiveType::I32,
        "i64" => PrimitiveType::I64,
        "i128" => PrimitiveType::I128,
        "actor" | "ActorId" => PrimitiveType::ActorId,
        "code" | "CodeId" => PrimitiveType::CodeId,
        "messageid" | "MessageId" => PrimitiveType::MessageId,
        "H160" => PrimitiveType::H160,
        "H256" => PrimitiveType::H256,
        "U256" => PrimitiveType::U256,
        _ => bail!("unexpected PrimitiveType {}", s),
    })
}

fn parse_type_decl(p: Pair<Rule>) -> Result<TypeDecl> {
    Ok(match p.as_rule() {
        // TypeDecl is `silent` Rule, but this for futureproof
        Rule::TypeDecl => parse_type_decl(p.into_inner().next().context("expected TypeDecl")?)?,
        Rule::Tuple => {
            let mut items = Vec::new();
            for el in p.into_inner() {
                items.push(parse_type_decl(el)?);
            }
            TypeDecl::Tuple(items)
        }
        Rule::Slice => {
            let mut it = p.into_inner();
            let ty = expect_next(&mut it, parse_type_decl)?;
            TypeDecl::Slice(Box::new(ty))
        }
        Rule::Array => {
            let mut it = p.into_inner();
            let ty = expect_next(&mut it, parse_type_decl)?;
            let len = expect_rule(&mut it, Rule::Number)?
                .as_str()
                .parse::<u32>()?;
            TypeDecl::Array {
                item: Box::new(ty),
                len,
            }
        }
        Rule::Option => {
            let mut it = p.into_inner();
            let ty = expect_next(&mut it, parse_type_decl)?;
            TypeDecl::Option(Box::new(ty))
        }
        Rule::Result => {
            let mut it = p.into_inner();
            let ok = expect_next(&mut it, parse_type_decl)?;
            let err = expect_next(&mut it, parse_type_decl)?;
            TypeDecl::Result {
                ok: Box::new(ok),
                err: Box::new(err),
            }
        }
        Rule::Primitive => TypeDecl::Primitive(map_primitive(p.as_str())?),
        Rule::PathType => {
            let mut path = String::new();
            let mut generics: Vec<TypeDecl> = Vec::new();
            for part in p.into_inner() {
                match part.as_rule() {
                    Rule::Path => path = part.as_str().to_string(),
                    Rule::Generics => {
                        for t in part.into_inner() {
                            generics.push(parse_type_decl(t)?);
                        }
                    }
                    _ => {}
                }
            }
            TypeDecl::UserDefined { path, generics }
        }
        other => bail!("unexpected rule in TypeDecl: {:?}", other),
    })
}

fn parse_param(p: Pair<'_, Rule>) -> Result<FuncParam> {
    let mut it = p.into_inner();
    let name = expect_next(&mut it, parse_ident)?;
    let ty = expect_next(&mut it, parse_type_decl)?;
    Ok(FuncParam {
        name,
        type_decl: ty,
    })
}

fn parse_field(p: Pair<'_, Rule>) -> Result<StructField, anyhow::Error> {
    let mut it = p.into_inner();
    let (docs, annotations) = parse_docs_and_annotations(&mut it)?;
    let part = it.next().context("expected Ident | TypeDecl")?;
    let (name, type_decl) = match part.as_rule() {
        Rule::Ident => {
            let name = part.as_str().to_string();
            let ty = expect_next(&mut it, parse_type_decl)?;
            (Some(name), ty)
        }
        _ => (None, parse_type_decl(part)?),
    };
    Ok(StructField {
        name,
        type_decl,
        annotations,
        docs,
    })
}

pub fn parse_type(p: Pair<Rule>) -> Result<Type> {
    match p.as_rule() {
        Rule::StructDecl => parse_struct_type(p),
        Rule::EnumDecl => parse_enum_type(p),
        Rule::AliasDecl => {
            todo!()
        }
        _ => bail!("expected StructDecl | EnumDecl | AliasDecl"),
    }
}

fn parse_struct_type(p: Pair<Rule>) -> Result<Type> {
    let mut it = p.into_inner();
    let (docs, annotations) = parse_docs_and_annotations(&mut it)?;
    let name = expect_next(&mut it, parse_ident)?;
    let mut type_params = Vec::new();
    let mut fields = Vec::new();
    for part in it {
        match part.as_rule() {
            Rule::TypeParams => {
                for i in part.into_inner().filter(|x| x.as_rule() == Rule::Ident) {
                    let name = i.as_str().to_string();
                    type_params.push(TypeParameter { name, ty: None });
                }
            }
            Rule::Fields => {
                for f in part.into_inner().filter(|x| x.as_rule() == Rule::Field) {
                    fields.push(parse_field(f)?);
                }
            }
            _ => bail!("expected StructDef | TupleDef | UnitDef"),
        };
    }

    Ok(Type {
        name,
        type_params,
        def: TypeDef::Struct(StructDef { fields }),
        docs,
        annotations,
    })
}

fn parse_enum_type(p: Pair<Rule>) -> Result<Type> {
    let mut it = p.into_inner();
    let (docs, annotations) = parse_docs_and_annotations(&mut it)?;
    let name = expect_next(&mut it, parse_ident)?;
    let mut type_params = Vec::new();
    let mut variants = Vec::new();
    for part in it {
        match part.as_rule() {
            Rule::TypeParams => {
                for i in part.into_inner().filter(|x| x.as_rule() == Rule::Ident) {
                    let name = i.as_str().to_string();
                    type_params.push(TypeParameter { name, ty: None });
                }
            }
            Rule::Variants => {
                for v in part.into_inner().filter(|x| x.as_rule() == Rule::Variant) {
                    variants.push(parse_enum_variant(v)?);
                }
            }
            _ => bail!("expected TypeParams | Variants"),
        };
    }

    Ok(Type {
        name,
        type_params,
        def: TypeDef::Enum(EnumDef { variants }),
        docs,
        annotations,
    })
}

fn parse_enum_variant(p: Pair<Rule>) -> Result<EnumVariant> {
    let mut it = p.into_inner();
    let (docs, annotations) = parse_docs_and_annotations(&mut it)?;
    let name = expect_next(&mut it, parse_ident)?;
    let mut fields = Vec::new();
    for part in it {
        match part.as_rule() {
            Rule::Fields => {
                for f in part.into_inner().filter(|x| x.as_rule() == Rule::Field) {
                    fields.push(parse_field(f)?);
                }
            }
            _ => bail!("expected Fields"),
        };
    }

    Ok(EnumVariant {
        name,
        def: StructDef { fields },
        docs,
        annotations,
    })
}

fn parse_func(p: Pair<Rule>) -> Result<ServiceFunc> {
    let mut it = p.into_inner();
    let (docs, annotations) = parse_docs_and_annotations(&mut it)?;
    let name = expect_next(&mut it, parse_ident)?;
    let is_query = annotations.iter().any(|(k, _)| k == "query");
    let mut params = Vec::new();
    let mut output = None;
    let mut throws = None;
    for part in it {
        match part.as_rule() {
            Rule::Params => {
                for p in part.into_inner().filter(|x| x.as_rule() == Rule::Param) {
                    params.push(parse_param(p)?);
                }
            }
            Rule::Ret => {
                output = Some(parse_type_decl(
                    part.into_inner().next().context("expect TypeDecl")?,
                )?)
            }
            Rule::Throws => {
                throws = Some(parse_type_decl(
                    part.into_inner().next().context("expect TypeDecl")?,
                )?)
            }
            _ => {}
        }
    }
    let output = output.unwrap_or(TypeDecl::Primitive(PrimitiveType::Void));
    Ok(ServiceFunc {
        name,
        params,
        output,
        throws,
        is_query,
        docs,
        annotations,
    })
}

fn parse_service(p: Pair<Rule>) -> Result<ServiceUnit> {
    let mut it = p.into_inner();
    let (docs, annotations) = parse_docs_and_annotations(&mut it)?;
    let name = expect_rule(&mut it, Rule::Ident)?.as_str().to_string();

    let mut extends = Vec::new();
    let mut events = Vec::new();
    let mut funcs = Vec::new();
    let mut types = Vec::new();
    for item in it {
        match item.as_rule() {
            Rule::ExtendsBlock => {
                for i in item.into_inner().filter(|x| x.as_rule() == Rule::Ident) {
                    extends.push(i.as_str().to_string());
                }
            }
            Rule::EventsBlock => {
                for e in item.into_inner().filter(|x| x.as_rule() == Rule::Variant) {
                    events.push(parse_enum_variant(e)?);
                }
            }
            Rule::FunctionsBlock => {
                for f in item.into_inner().filter(|x| x.as_rule() == Rule::FuncDecl) {
                    funcs.push(parse_func(f)?);
                }
            }
            Rule::TypesBlock => {
                for t in item.into_inner() {
                    types.push(parse_type(t)?);
                }
            }
            _ => {}
        }
    }

    Ok(ServiceUnit {
        name,
        extends,
        events,
        funcs,
        types,
        docs,
        annotations,
    })
}

fn parse_ctor_func(p: Pair<Rule>) -> Result<CtorFunc> {
    let mut it = p.into_inner();
    let (docs, annotations) = parse_docs_and_annotations(&mut it)?;
    let name = expect_rule(&mut it, Rule::Ident)?.as_str().to_string();
    let mut params = Vec::new();
    for part in it {
        match part.as_rule() {
            Rule::Params => {
                for p in part.into_inner().filter(|x| x.as_rule() == Rule::Param) {
                    params.push(parse_param(p)?);
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
    let (docs, annotations) = parse_docs_and_annotations(&mut it)?;
    let name = expect_next(&mut it, parse_ident)?;
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
                    let mut sit = s.into_inner();
                    let (docs, annotations) = parse_docs_and_annotations(&mut sit)?;
                    let mut name = expect_next(&mut sit, parse_ident)?;
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
                for t in item.into_inner() {
                    types.push(parse_type(t)?);
                }
            }
            _ => bail!("expected ConstructorsBlock | ServicesBlock | TypesBlock"),
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

fn parse_docs_and_annotations(
    pairs: &mut Pairs<Rule>,
) -> Result<(Vec<String>, Vec<(String, Option<String>)>)> {
    let mut docs = Vec::new();
    let mut anns = Vec::new();
    // iter over cloned
    for p in pairs.clone() {
        // peek Docs or Anns
        match p.as_rule() {
            Rule::DocLine => {
                // pop pair
                let _ = pairs.next();
                for d in p.into_inner() {
                    match d.as_rule() {
                        Rule::StrToEol => docs.push(d.as_str().trim().to_string()),
                        _ => {}
                    }
                }
            }
            Rule::LocalAnn => {
                // pop pair
                let _ = pairs.next();
                anns.push(parse_annotation(p)?);
            }
            _ => break,
        }
    }
    Ok((docs, anns))
}

fn expect_next<'a, F: FnOnce(Pair<'a, Rule>) -> Result<T>, T>(
    it: &mut impl Iterator<Item = Pair<'a, Rule>>,
    f: F,
) -> Result<T> {
    if let Some(p) = it.next() {
        return f(p);
    }
    bail!("expected next Rule")
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

// ------------------------------ Tests ----------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_docs_and_annotations_lines() {
        const SRC: &str = r#"
            // Comment
            /// Defines status of some point as colored by somebody or dead for some reason.
            /// Dead point - won't be available for coloring anymore.
            @indexed
            "#;
        let mut pairs = IdlParser::parse(Rule::DocsAndAnnotations, SRC).expect("parse idl");
        println!("pairs: {pairs:?}");
        let (docs, anns) = parse_docs_and_annotations(&mut pairs).expect("parse annotations");
        println!("docs: {docs:?}");
        assert_eq!(
            docs,
            vec![
                "Defines status of some point as colored by somebody or dead for some reason.",
                "Dead point - won't be available for coloring anymore."
            ],
        );
        assert_eq!(anns, vec![("indexed".to_string(), None)])
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
    fn parse_vector_of_tuples() {
        use PrimitiveType::*;
        use TypeDecl::*;

        const SRC: &str = r#"[(Point<u32>, Option<PointStatus>, u32)]"#;
        let mut pairs = IdlParser::parse(Rule::TypeDecl, SRC).expect("parse idl");
        let ty = expect_next(&mut pairs, parse_type_decl).expect("parse TypeDecl");
        println!("ty: {ty:?}");
        assert_eq!(
            ty,
            Slice(Box::new(Tuple(vec![
                UserDefined {
                    path: "Point".to_string(),
                    generics: vec![Primitive(U32)]
                },
                Option(Box::new(UserDefined {
                    path: "PointStatus".to_string(),
                    generics: vec![]
                })),
                Primitive(U32)
            ])))
        );
    }

    #[test]
    fn pars_service_func() {
        use PrimitiveType::*;
        use TypeDecl::*;

        const SRC: &str = r#"
            /// Sets color for the point.
            /// app -> `fn color_point(&mut self, point: Point<u32>, color: Color) -> Result<(), ColorError>`
            /// On `Ok` - auto-reply. On `Err` -> app will encode error bytes of `ColorError` (`gr_panic_bytes`).
            ColorPoint(point: (u32, u32), color: Color) throws ColorError;"#;
        let mut pairs = IdlParser::parse(Rule::FuncDecl, SRC).expect("parse idl");
        let func = parse_func(pairs.next().expect("FuncDecl")).expect("parse func");
        println!("func: {func:?}");
        assert_eq!(
            func,
            ServiceFunc {
                name: "ColorPoint".to_string(),
                params: vec![
                    FuncParam { name: "point".to_string(), type_decl: Tuple(vec![Primitive(U32), Primitive(U32)]) },
                    FuncParam { name: "color".to_string(), type_decl: UserDefined { path: "Color".to_string(), generics: vec![] } }
                ],
                output: Primitive(Void),
                throws: Some(UserDefined { path: "ColorError".to_string(), generics: vec![] }),
                is_query: false,
                annotations: vec![],
                docs: vec![
                    "Sets color for the point.".to_string(),
                    "app -> `fn color_point(&mut self, point: Point<u32>, color: Color) -> Result<(), ColorError>`".to_string(),
                    "On `Ok` - auto-reply. On `Err` -> app will encode error bytes of `ColorError` (`gr_panic_bytes`).".to_string(),
                ],
            }
        );
    }

    #[test]
    fn parse_minimal_service() {
        const SRC: &str = r#"
            /// Example
            service X {
                functions { Ping() -> bool; }
                events { E }
                types { struct T; }
            }
        "#;
        let mut pairs = IdlParser::parse(Rule::ServiceDecl, SRC).expect("parse idl");
        let svc = expect_next(&mut pairs, parse_service).expect("parse");
        assert_eq!(svc.name, "X");
        assert!(svc.funcs.iter().any(|f| f.name == "Ping"));
    }

    #[test]
    fn parse_test_idl() {
        const SRC: &str = r#"
!@sails: 0.1.0
!@include: ownable.idl
!@include: git://github.com/some_repo/tippable.idl

/// Canvas service
service Canvas {
    // Merge `functions`, `events`, `types`, from Ownable, Tippable and Pausable services
    extends {
        Ownable,
        Tippable,
        Pausable,
    }

    // Canvas service events
    events {
        StatusChanged(Point),
        Jubilee {
            /// Amount of alive points.
            @indexed
            @nonzero
            amount: u64,
            bits: bitvec,
        },
        E1,
    }

    functions {
        /// Sets color for the point.
        /// app -> `fn color_point(&mut self, point: Point<u32>, color: Color) -> Result<(), ColorError>`
        /// On `Ok` - auto-reply. On `Err` -> app will encode error bytes of `ColorError` (`gr_panic_bytes`).
        ColorPoint(point: Point, color: Color) throws ColorError;

        /// Kills the point.
        /// app -> `fn kill_point(&mut self, point: Point) -> Result<bool, String>`
        KillPoint(point: Point) -> bool throws String;

        /// Returns known points.
        /// app -> `fn points(&self, ...) -> Result<BTreeMap<Point, PointStatus>, String>`
        @query
        Points(offset: u32, len: u32) -> [(Point, PointStatus)] throws string;

        /// Returns status set for given point.
        @query
        PointStatus(point: Point) -> PointStatus;
    }

    types {
        // Point with two coordinates.
        struct Point<T> {
            /// Horizontal coordinate.
            x: T,
            /// Vertical coordinate.
            y: T,
        }

        struct Color {
            color: [u8; 4],
            space: ColorSpace,
        }

        enum ColorSpace {
            RGB,
            HSV,
            CMYK,
        }

        /// Defines status of some point as colored by somebody or dead for some reason.
        enum PointStatus {
            /// Colored into some RGB.
            Colored {
                /// Who has colored it.
                author: actor,
                /// Color used.
                color: Color,
            },
            /// Dead point - won't be available for coloring anymore.
            Dead,
        }

        /// Error happened during point setting.
        enum ColorError {
            InvalidSource,
            DeadPoint,
        }
    }
}

/// Pausable Service
service Pausable {
    events {
        Paused,
        Unpaused,
    }

    functions {
        // Client: `fn pause(&mut self) -> Result<(), SailsEnvError>`
        Pause();
        Unpause();
    }

    types {
        struct PausedError;
    }
}
        "#;
        let doc = IdlDoc::parse(SRC).expect("parse idl");
        println!("{:#?}", doc);
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
        println!("{:#?}", doc);
        assert_eq!(3, doc.globals.len());
    }
}
