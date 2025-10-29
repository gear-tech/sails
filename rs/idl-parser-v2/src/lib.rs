// Sails IDL v0.3 — parser using `nom` (nom 8 compatible)

use nom::{
    IResult, Parser,
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while, take_while1},
    character::complete::{char, digit1, line_ending, multispace0, not_line_ending},
    combinator::{all_consuming, map, map_res, not, opt, peek, recognize, value},
    multi::{many0, separated_list0, separated_list1},
    sequence::{delimited, preceded, separated_pair, terminated},
};
use std::str::FromStr;

// -------------------------------- Target model ---------------------------------

/// A structure describing program
#[derive(Debug, Default, PartialEq, Clone)]
pub struct IdlUnit {
    pub globals: Vec<(String, Option<String>)>,
    pub program: Option<ProgramUnit>,
    pub services: Vec<ServiceUnit>,
}

/// A structure describing program
#[derive(Debug, Default, PartialEq, Clone)]
pub struct ProgramUnit {
    pub name: String,
    pub ctors: Vec<CtorFunc>,
    pub services: Vec<(String, String)>,
    pub types: Vec<Type>,
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}

/// A structure describing one of program constructor functions
#[derive(Debug, PartialEq, Clone)]
pub struct CtorFunc {
    pub name: String,
    pub params: Vec<FuncParam>,
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ServiceUnit {
    pub name: String,
    pub extends: Vec<String>,
    pub funcs: Vec<ServiceFunc>,
    pub events: Vec<ServiceEvent>,
    pub types: Vec<Type>,
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ServiceFunc {
    pub name: String,
    pub params: Vec<FuncParam>,
    pub output: TypeDecl,
    pub throws: Option<TypeDecl>,
    pub is_query: bool,
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FuncParam {
    pub name: String,
    pub type_decl: TypeDecl,
}

pub type ServiceEvent = EnumVariant;

#[derive(Debug, PartialEq, Clone)]
pub struct Type {
    pub name: String,
    pub type_params: Vec<TypeParameter>,
    pub def: TypeDef,
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypeParameter {
    /// The name of the generic type parameter e.g. "T".
    pub name: String,
    /// The concrete type for the type parameter.
    ///
    /// `None` if the type parameter is skipped.
    pub ty: Option<TypeDecl>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TypeDecl {
    Slice(Box<TypeDecl>),
    Array {
        item: Box<TypeDecl>,
        len: u32,
    },
    Tuple(Vec<TypeDecl>),
    // Map {
    //     key: Box<TypeDecl>,
    //     value: Box<TypeDecl>,
    // },
    Option(Box<TypeDecl>),
    Result {
        ok: Box<TypeDecl>,
        err: Box<TypeDecl>,
    },
    Primitive(PrimitiveType),
    UserDefined(String),
    // Def(TypeDef),
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[repr(u8)]
pub enum PrimitiveType {
    Null,
    Bool,
    Char,
    String,
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    ActorId,
    CodeId,
    MessageId,
    H256,
    U256,
    H160,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TypeDef {
    Struct(StructDef),
    Enum(EnumDef),
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructDef {
    pub fields: Vec<StructField>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructField {
    pub name: Option<String>,
    pub type_decl: TypeDecl,
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct EnumDef {
    pub variants: Vec<EnumVariant>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub def: StructDef,
    pub docs: Vec<String>,
    pub annotations: Vec<(String, Option<String>)>,
}

// ------------------------- Lexing helpers -------------------------

type Res<'a, O> = IResult<&'a str, O>;

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}
fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn tag_ident<'a>(
    i: &'a str,
) -> impl Parser<&'a str, Output = &'a str, Error = nom::error::Error<&'a str>> {
    recognize(terminated(
        tag_no_case(i),
        not(peek(take_while1(is_ident_char))),
    ))
}

fn ws<'a, O, P>(inner: P) -> impl Parser<&'a str, Output = O, Error = nom::error::Error<&'a str>>
where
    P: Parser<&'a str, Output = O, Error = nom::error::Error<&'a str>>,
{
    delimited(space_or_comments, inner, space_or_comments)
}

fn space_or_comments(i: &str) -> Res<'_, ()> {
    let mut input = i;
    loop {
        let (rest, _) = multispace0(input)?;
        input = rest;
        // line comments "// ..."
        if let Ok((rest, _)) = terminated(
            preceded(
                (tag::<&str, &str, ()>(r"//"), not(peek(char('/')))),
                take_while(|c| c != '\n'),
            ),
            opt(line_ending),
        )
        .parse(input)
        {
            input = rest;
            continue;
        }
        break;
    }
    Ok((input, ()))
}

fn doc_lines(i: &str) -> Res<'_, Vec<String>> {
    many0(map(
        terminated(
            preceded(ws(tag("///")), take_while(|c| c != '\n')),
            opt(line_ending),
        ),
        |s: &str| s.trim().to_string(),
    ))
    .parse(i)
}

/// Parse lines of form `@key` or `@key: value` into HashMap<String, Option<String>>
fn annotations(i: &str) -> Res<'_, Vec<(String, Option<String>)>> {
    many0(annotation_line).parse(i)
}

/// Parse line of form `@key` or `@key: value` into (String, Option<String>)
fn annotation_line(i: &str) -> Res<'_, (String, Option<String>)> {
    map(
        terminated(
            (
                ws(preceded(char('@'), ident)),
                opt(preceded(ws(char(':')), ws(not_line_ending))),
            ),
            opt(line_ending),
        ),
        |(key, value)| (key, value.map(|v| v.trim().to_string())),
    )
    .parse(i)
}

/// Parse line of form `!@key` or `!@key: value` into (<String, Option<String>)
fn global_line(i: &str) -> Res<'_, (String, Option<String>)> {
    map(
        terminated(
            (
                ws(preceded(tag("!@"), ident)),
                opt(preceded(ws(char(':')), ws(not_line_ending))),
            ),
            opt(line_ending),
        ),
        |(key, value)| (key, value.map(|v| v.trim().to_string())),
    )
    .parse(i)
}

fn ident(i: &str) -> Res<'_, String> {
    map(
        recognize((take_while1(is_ident_start), take_while(is_ident_char))),
        |s: &str| s.to_string(),
    )
    .parse(i)
}

fn number_u32(i: &str) -> Res<'_, u32> {
    map_res(ws(digit1), |s: &str| u32::from_str(s)).parse(i)
}

fn sep_comma<'a, O, P>(
    inner: P,
) -> impl Parser<&'a str, Output = Vec<O>, Error = nom::error::Error<&'a str>>
where
    P: Parser<&'a str, Output = O, Error = nom::error::Error<&'a str>>,
{
    terminated(separated_list0(ws(char(',')), inner), opt(ws(char(','))))
}

fn path_ident(i: &str) -> Res<'_, String> {
    map(recognize(separated_list1(tag("::"), ident)), |s: &str| {
        s.to_string()
    })
    .parse(i)
}

// ------------------------- Type parser -------------------------

fn primitive(i: &str) -> Res<'_, PrimitiveType> {
    ws(alt((
        value(PrimitiveType::Bool, tag_ident("bool")),
        value(PrimitiveType::Char, tag_ident("char")),
        value(PrimitiveType::String, tag_ident("string")),
        value(PrimitiveType::U8, tag_ident("u8")),
        value(PrimitiveType::U16, tag_ident("u16")),
        value(PrimitiveType::U32, tag_ident("u32")),
        value(PrimitiveType::U64, tag_ident("u64")),
        value(PrimitiveType::U128, tag_ident("u128")),
        value(PrimitiveType::I8, tag_ident("i8")),
        value(PrimitiveType::I16, tag_ident("i16")),
        value(PrimitiveType::I32, tag_ident("i32")),
        value(PrimitiveType::I64, tag_ident("i64")),
        value(PrimitiveType::I128, tag_ident("i128")),
        value(PrimitiveType::ActorId, tag_ident("ActorId")),
        value(PrimitiveType::ActorId, tag_ident("actor")),
        value(PrimitiveType::CodeId, tag_ident("CodeId")),
        value(PrimitiveType::CodeId, tag_ident("code")),
        value(PrimitiveType::MessageId, tag_ident("messageid")),
        value(PrimitiveType::H256, tag_ident("h256")),
        value(PrimitiveType::U256, tag_ident("u256")),
        value(PrimitiveType::H160, tag_ident("h160")),
    )))
    .parse(i)
}

fn type_slice(i: &str) -> Res<'_, TypeDecl> {
    map(
        delimited(ws(char('[')), ws(type_decl), ws(char(']'))),
        |t| TypeDecl::Slice(Box::new(t)),
    )
    .parse(i)
}

fn type_array(i: &str) -> Res<'_, TypeDecl> {
    map(
        delimited(
            ws(char('[')),
            separated_pair(ws(type_decl), ws(char(';')), number_u32),
            ws(char(']')),
        ),
        |(t, len)| TypeDecl::Array {
            item: Box::new(t),
            len,
        },
    )
    .parse(i)
}

fn type_tuple(i: &str) -> Res<'_, TypeDecl> {
    map(
        delimited(ws(char('(')), sep_comma(type_decl), ws(char(')'))),
        TypeDecl::Tuple,
    )
    .parse(i)
}

fn type_primitive(i: &str) -> Res<'_, TypeDecl> {
    map(primitive, |p| TypeDecl::Primitive(p)).parse(i)
}

fn type_user_defined(i: &str) -> Res<'_, TypeDecl> {
    map(path_ident, |p| TypeDecl::UserDefined(p)).parse(i)
}

fn struct_field(i: &str) -> Res<'_, StructField> {
    let (i, docs) = ws(doc_lines).parse(i)?;
    let (i, annotations) = ws(annotations).parse(i)?;
    let (i, name_opt) = opt(terminated(ws(ident), ws(char(':')))).parse(i)?;
    let (i, td) = ws(type_decl).parse(i)?;
    Ok((
        i,
        StructField {
            name: name_opt,
            type_decl: td,
            docs,
            annotations,
        },
    ))
}

fn struct_def(i: &str) -> Res<'_, StructDef> {
    map(
        terminated(
            delimited(ws(char('{')), sep_comma(struct_field), ws(char('}'))),
            opt(ws(char(';'))),
        ),
        |fields| StructDef { fields },
    )
    .parse(i)
}

fn tuple_struct_def(i: &str) -> Res<'_, StructDef> {
    map(
        terminated(
            delimited(ws(char('(')), sep_comma(type_decl), ws(char(')'))),
            opt(ws(char(';'))),
        ),
        |items| StructDef {
            fields: items
                .into_iter()
                .map(|t| StructField {
                    name: None,
                    type_decl: t,
                    docs: Default::default(),
                    annotations: Default::default(),
                })
                .collect(),
        },
    )
    .parse(i)
}

fn unit_struct_def(i: &str) -> Res<'_, StructDef> {
    value(StructDef { fields: vec![] }, opt(ws(char(';')))).parse(i)
}

/// struct-like, tuple-like or unit-like variants
fn any_struct_def(i: &str) -> Res<'_, StructDef> {
    ws(alt((struct_def, tuple_struct_def, unit_struct_def))).parse(i)
}

fn enum_variant(i: &str) -> Res<'_, EnumVariant> {
    let (i, docs) = ws(doc_lines).parse(i)?;
    let (i, annotations) = ws(annotations).parse(i)?;
    let (i, name) = ws(ident).parse(i)?;
    let (i, def) = any_struct_def.parse(i)?;
    Ok((
        i,
        EnumVariant {
            name,
            def,
            docs,
            annotations,
        },
    ))
}

fn enum_def(i: &str) -> Res<'_, TypeDef> {
    map(
        delimited(ws(char('{')), sep_comma(enum_variant), ws(char('}'))),
        |variants| TypeDef::Enum(EnumDef { variants }),
    )
    .parse(i)
}

// fn type_def(i: &str) -> Res<'_, TypeDef> {
//     alt((struct_def, enum_def)).parse(i)
// }

// fn type_decl_struct(i: &str) -> Res<'_, TypeDecl> {
//     map(struct_def, TypeDecl::Def).parse(i)
// }
// fn type_decl_enum(i: &str) -> Res<'_, TypeDecl> {
//     map(enum_def, TypeDecl::Def).parse(i)
// }

fn type_option(i: &str) -> Res<'_, TypeDecl> {
    map(
        preceded(
            ws(tag_no_case("Option")),
            delimited(ws(char('<')), ws(type_decl), ws(char('>'))),
        ),
        |td| TypeDecl::Option(Box::new(td)),
    )
    .parse(i)
}

fn type_decl(i: &str) -> Res<'_, TypeDecl> {
    ws(alt((
        type_array,
        type_slice,
        type_tuple,
        // type_map,
        type_option,
        // type_result,
        // inline defs
        // type_decl_struct,
        // type_decl_enum,
        type_primitive,
        type_user_defined,
    )))
    .parse(i)
}

fn type_item(i: &str) -> Res<'_, Type> {
    let (i, docs) = ws(doc_lines).parse(i)?;
    let (i, annotations) = ws(annotations).parse(i)?;
    let (i, kind) = ws(alt((tag("struct"), tag("enum")))).parse(i)?;
    let (i, name) = ws(ident).parse(i)?;
    // Optional generics <...>
    let (i, type_params) = ws(opt(delimited(
        char('<'),
        sep_comma(map(ident, |name| TypeParameter { name, ty: None })),
        char('>'),
    )))
    .parse(i)?;
    let (i, def) = if kind == "struct" {
        map(any_struct_def, TypeDef::Struct).parse(i)?
    } else {
        ws(enum_def).parse(i)?
    };
    Ok((
        i,
        Type {
            name,
            type_params: type_params.unwrap_or_default(),
            def,
            docs,
            annotations,
        },
    ))
}

// ------------------------- Service parser -------------------------

fn func_param(i: &str) -> Res<'_, FuncParam> {
    let (i, name) = ws(ident).parse(i)?;
    let (i, _) = ws(char(':')).parse(i)?;
    let (i, td) = ws(type_decl).parse(i)?;
    Ok((
        i,
        FuncParam {
            name,
            type_decl: td,
        },
    ))
}

fn param_list(i: &str) -> Res<'_, Vec<FuncParam>> {
    delimited(ws(char('(')), sep_comma(func_param), ws(char(')'))).parse(i)
}

fn throws_type(i: &str) -> Res<'_, TypeDecl> {
    preceded(ws(tag_no_case("throws")), ws(type_decl)).parse(i)
}

fn service_func(i: &str) -> Res<'_, ServiceFunc> {
    let (i, docs) = ws(doc_lines).parse(i)?;
    let (i, annotations) = ws(annotations).parse(i)?;
    let is_query = annotations.iter().any(|(k, _)| k == "query");
    let (i, name) = ws(ident).parse(i)?;
    let (i, params) = ws(param_list).parse(i)?;
    // return type optional
    let (i, output) = opt(preceded(ws(tag("->")), ws(type_decl))).parse(i)?;
    let output = output.unwrap_or(TypeDecl::Primitive(PrimitiveType::Null));
    let (i, throws) = opt(ws(throws_type)).parse(i)?;
    let (i, _) = ws(char(';')).parse(i)?;
    Ok((
        i,
        ServiceFunc {
            name,
            params,
            output,
            throws,
            is_query,
            docs,
            annotations,
        },
    ))
}

fn extends_list(i: &str) -> Res<'_, Vec<String>> {
    delimited(
        ws(tag("extends")),
        delimited(ws(char('{')), sep_comma(ident), ws(char('}'))),
        space_or_comments,
    )
    .parse(i)
}

fn event_list(i: &str) -> Res<'_, Vec<ServiceEvent>> {
    preceded(
        ws(tag("events")),
        delimited(ws(char('{')), sep_comma(enum_variant), ws(char('}'))),
    )
    .parse(i)
}

fn type_list(i: &str) -> Res<'_, Vec<Type>> {
    delimited(
        ws(tag("types")),
        delimited(ws(char('{')), many0(ws(type_item)), ws(char('}'))),
        space_or_comments,
    )
    .parse(i)
}

fn function_list(i: &str) -> Res<'_, Vec<ServiceFunc>> {
    delimited(
        ws(tag("functions")),
        delimited(ws(char('{')), many0(ws(service_func)), ws(char('}'))),
        space_or_comments,
    )
    .parse(i)
}

fn service_unit(i: &str) -> Res<'_, ServiceUnit> {
    let (i, docs) = ws(doc_lines).parse(i)?;
    let (i, annotations) = ws(annotations).parse(i)?;
    let (i, name) = ws(preceded(tag("service"), ws(ident))).parse(i)?;
    let (i, _) = ws(char('{')).parse(i)?;

    let mut extends = vec![];
    let mut events = vec![];
    let mut funcs = vec![];
    let mut types = vec![];
    let mut i = i;
    loop {
        // extends
        if let Ok((rest, res)) = ws(extends_list).parse(i) {
            i = rest;
            extends = res;
            continue;
        }
        // events
        if let Ok((rest, res)) = ws(event_list).parse(i) {
            i = rest;
            events = res;
            continue;
        }
        // functions
        if let Ok((rest, res)) = ws(function_list).parse(i) {
            i = rest;
            funcs = res;
            continue;
        }
        // types
        if let Ok((rest, res)) = ws(type_list).parse(i) {
            i = rest;
            types = res;
            continue;
        }
        break;
    }

    let (i, _) = ws(char('}')).parse(i)?;

    Ok((
        i,
        ServiceUnit {
            name,
            extends,
            funcs,
            events,
            types,
            docs,
            annotations,
        },
    ))
}

// Parse the first service block after skipping headers/annotations
pub fn parse_service(i: &str) -> Res<'_, ServiceUnit> {
    ws(service_unit).parse(i)
}

// ------------------------- Program parser -------------------------

fn ctor_func(i: &str) -> Res<CtorFunc> {
    let (i, docs) = ws(doc_lines).parse(i)?;
    let (i, annotations) = ws(annotations).parse(i)?;
    let (i, name) = ws(ident).parse(i)?;
    let (i, params) = ws(param_list).parse(i)?;
    let (i, _throws) = opt(ws(throws_type)).parse(i)?;
    let (i, _) = ws(char(';')).parse(i)?;
    Ok((
        i,
        CtorFunc {
            name,
            params,
            docs,
            annotations,
        },
    ))
}

fn service_expo(i: &str) -> Res<'_, (String, String)> {
    map(
        (ws(ident), opt(preceded(ws(char(':')), ws(ident)))),
        |(first, second)| {
            (
                first.trim().to_string(),
                second.unwrap_or(first).trim().to_string(),
            )
        },
    )
    .parse(i)
}

fn ctor_list(i: &str) -> Res<'_, Vec<CtorFunc>> {
    delimited(
        ws(tag("constructors")),
        delimited(ws(char('{')), many0(ws(ctor_func)), ws(char('}'))),
        space_or_comments,
    )
    .parse(i)
}

fn service_list(i: &str) -> Res<'_, Vec<(String, String)>> {
    preceded(
        ws(tag("services")),
        delimited(ws(char('{')), sep_comma(service_expo), ws(char('}'))),
    )
    .parse(i)
}

fn program_unit(i: &str) -> Res<ProgramUnit> {
    let (i, docs) = ws(doc_lines).parse(i)?;
    let (i, annotations) = ws(annotations).parse(i)?;
    let (i, name) = ws(preceded(tag("program"), ws(ident))).parse(i)?;

    let (i, _) = ws(char('{')).parse(i)?;

    let mut ctors = vec![];
    let mut services = vec![];
    let mut types = vec![];
    let mut i = i;
    loop {
        // constructors
        if let Ok((rest, res)) = ws(ctor_list).parse(i) {
            i = rest;
            ctors = res;
            continue;
        }
        // services
        if let Ok((rest, res)) = ws(service_list).parse(i) {
            i = rest;
            services = res;
            continue;
        }
        // types
        if let Ok((rest, res)) = ws(type_list).parse(i) {
            i = rest;
            types = res;
            continue;
        }
        break;
    }

    let (i, _) = ws(char('}')).parse(i)?;

    Ok((
        i,
        ProgramUnit {
            name,
            ctors,
            services,
            types,
            docs,
            annotations,
        },
    ))
}

fn idl(mut i: &str) -> Res<'_, IdlUnit> {
    let mut globals = Vec::new();
    let mut services = Vec::new();
    let mut program = None;
    i = space_or_comments(i)?.0;
    loop {
        if let Ok((rest, (k, v))) = ws(global_line).parse(i) {
            globals.push((k, v));
            i = rest;
            continue;
        }
        if let Ok((rest, svc)) = ws(service_unit).parse(i) {
            services.push(svc);
            i = rest;
            continue;
        }
        if let Ok((rest, prg)) = ws(program_unit).parse(i) {
            program = Some(prg);
            i = rest;
            continue;
        }
        break;
    }

    Ok((
        i,
        IdlUnit {
            globals,
            program,
            services,
        },
    ))
}

pub fn parse_idl(i: &str) -> Res<'_, IdlUnit> {
    all_consuming(ws(idl)).parse(i)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn parse_doc_lines() {
        const SRC: &str = r#"
            /// Defines status of some point as colored by somebody or dead for some reason.
            /// Dead point - won't be available for coloring anymore."#;
        let (rest, res) = ws(doc_lines).parse(SRC).expect("parse");
        println!("res: {res:?}, rest: {rest}");
        assert_eq!(2, res.len());
        assert!(rest.trim().is_empty());
    }

    #[test]
    fn parse_annotations() {
        const SRC: &str = r#"
            @key1
            @key2: value2"#;
        let (rest, res) = ws(annotations).parse(SRC).expect("parse");
        println!("res: {res:?}, rest: {rest}");
        assert_eq!(2, res.len());
        assert!(rest.trim().is_empty());
    }

    #[test]
    fn parse_vector_of_tuples() {
        const SRC: &str = r#"[(Point, PointStatus)]"#;
        let (rest, res) = ws(type_slice).parse(SRC).expect("parse");
        println!("res: {res:?}, rest: {rest}");
        assert!(rest.trim().is_empty());
    }

    #[test]
    fn parse_option() {
        const SRC: &str = r#"Option<(String, PointStatus)>"#;
        let (rest, res) = ws(type_option).parse(SRC).expect("parse");
        println!("res: {res:?}, rest: {rest}");
        assert!(rest.trim().is_empty());
    }

    #[test]
    fn parse_struct_def() {
        const SRC: &str = r#"{
            /// Who has colored it.
            @indexed
            author: actor,
            /// Color used.
            color: Color,
        }"#;
        let (rest, res) = any_struct_def.parse(SRC).expect("parse");
        println!("res: {res:?}, rest: {rest}");
        assert!(rest.trim().is_empty());

        let (rest, res) = any_struct_def
            .parse(r#"(ActorId, ActorColor)"#)
            .expect("parse");
        println!("res: {res:?}, rest: {rest}");
        assert!(rest.trim().is_empty());

        let (rest, res) = any_struct_def.parse(r#"(u32,)"#).expect("parse");
        println!("res: {res:?}, rest: {rest}");
        assert!(rest.trim().is_empty());

        let (rest, res) = any_struct_def.parse(r#""#).expect("parse");
        println!("res: {res:?}, rest: {rest}");
        assert!(rest.trim().is_empty());
    }

    #[test]
    fn parse_enum() {
        const SRC: &str = r#"
            /// Defines status of some point as colored by somebody or dead for some reason.
            enum PointStatus {
                Transparent(actor),
                /// Colored into some RGB.
                Colored {
                    /// Who has colored it.
                    author: actor,
                    /// Color used.
                    color: Color,
                },
                /// Dead point - won't be available for coloring anymore.
                Dead,
            }"#;
        let (rest, res) = ws(type_item).parse(SRC).expect("parse");
        println!("res: {res:?}, rest: {rest}");
        assert!(rest.trim().is_empty());
    }

    #[test]
    fn parse_func() {
        const SRC: &str = r#"
            /// Sets color for the point.
            /// app -> `fn color_point(&mut self, point: Point<u32>, color: Color) -> Result<(), ColorError>`
            /// On `Ok` - auto-reply. On `Err` -> app will encode error bytes of `ColorError` (`gr_panic_bytes`).
            ColorPoint(point: (u32, u32), color: Color) throws ColorError;"#;
        let (rest, res) = ws(service_func).parse(SRC).expect("parse");
        println!("res: {res:?}, rest: {rest}");
        assert!(rest.trim().is_empty());
    }

    #[test]
    fn parse_single_service() {
        const SRC: &str = r#"
            /// Pausable Service
            service Pausable {
                events {
                    Paused,
                    Unpaused,
                }

                functions {
                    /// Pause func
                    // Client: `fn pause(&mut self) -> Result<(), SailsEnvError>`
                    Pause();
                    Unpause();
                }

                types {
                    struct PausedError;
                }
            }"#;
        let (rest, res) = parse_service.parse(SRC).expect("parse");
        println!("res: {res:?}, rest: {rest}");
        assert!(rest.trim().is_empty());
    }

    #[test]
    fn parse_multi_services() {
        let (rest, res) = idl(SRC).expect("parse");
        println!("res: {res:?}, rest: {rest}");
        assert!(rest.trim().is_empty());
        assert_eq!(res.services.len(), 2);

        let canvas = &res.services[0];
        assert_eq!(canvas.name, "Canvas");
        assert_eq!(canvas.extends, vec!["Ownable", "Tippable", "Pausable"]);
        assert!(
            canvas
                .funcs
                .iter()
                .any(|f| f.name == "Points" && f.is_query)
        );
        assert!(canvas.events.iter().any(|e| e.name == "Jubilee"));
        assert!(canvas.types.iter().any(|t| t.name == "Point"));

        let paus = &res.services[1];
        assert_eq!(paus.name, "Pausable");
        assert!(paus.funcs.iter().any(|f| f.name == "Pause"));
        assert!(paus.types.iter().any(|t| t.name == "PausedError"));
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
        let (rest, res) = idl.parse(SRC).expect("parse");
        println!("res: {res:?}, rest: {rest}");
        assert!(rest.trim().is_empty());
    }
}
