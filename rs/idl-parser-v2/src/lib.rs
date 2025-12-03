#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec::Vec,
};
pub use sails_idl_meta as ast;

pub mod error;
pub mod ffi {
    pub mod ast;
}
mod post_process;
pub mod visitor;

// Sails IDL v2 — parser using `pest-rs`
use crate::error::{Error, Result, RuleError};
use core::str::FromStr;
use pest::Parser;
use pest::iterators::{Pair, Pairs};
use sails_idl_meta::*;

type Annotation = (String, Option<String>);

#[derive(pest_derive::Parser)]
#[grammar = "idl.pest"]
pub struct IdlParser;

// ----------------------------- Public API ------------------------------------
pub fn parse_idl(src: &str) -> Result<IdlDoc> {
    let mut pairs = IdlParser::parse(Rule::Top, src)?;
    let mut doc = build_idl(
        pairs
            .next()
            .ok_or(Error::Rule(RuleError::Expected("expected Top".to_string())))?,
    )?;

    post_process::validate_and_post_process(&mut doc)?;
    Ok(doc)
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
            Rule::ProgramDecl => {
                if program.replace(parse_program(p)?).is_some() {
                    return Err(Error::ValidationError(
                        "Expected at most one program per IDL document".to_string(),
                    ));
                }
            }
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
    Err(Error::Rule(RuleError::Expected(
        "expected Ident".to_string(),
    )))
}

fn parse_annotation(p: Pair<Rule>) -> Result<Annotation> {
    let mut key = None;
    let mut val = None;
    for i in p.into_inner() {
        match i.as_rule() {
            Rule::Ident => key = Some(i.as_str().trim().to_string()),
            Rule::StrToEol => val = Some(i.as_str().trim().to_string()),
            _ => {}
        }
    }
    let key = key.ok_or(Error::Rule(RuleError::Expected(
        "expected Ident".to_string(),
    )))?;
    Ok((key, val))
}

fn parse_type_decl(p: Pair<Rule>) -> Result<TypeDecl> {
    Ok(match p.as_rule() {
        // TypeDecl is `silent` Rule, but this for futureproof
        Rule::TypeDecl => parse_type_decl(p.into_inner().next().ok_or(Error::Rule(
            RuleError::Expected("expected TypeDecl".to_string()),
        ))?)?,
        Rule::Tuple => {
            let mut types = Vec::new();
            for el in p.into_inner() {
                types.push(parse_type_decl(el)?);
            }
            if types.is_empty() {
                TypeDecl::Primitive(PrimitiveType::Void)
            } else {
                TypeDecl::Tuple { types }
            }
        }
        Rule::Slice => {
            let mut it = p.into_inner();
            let ty = expect_next(&mut it, parse_type_decl)?;
            TypeDecl::Slice { item: Box::new(ty) }
        }
        Rule::Array => {
            let mut it = p.into_inner();
            let ty = expect_next(&mut it, parse_type_decl)?;
            let len = expect_rule(&mut it, Rule::Number)?
                .as_str()
                .parse::<u32>()
                .map_err(|e| Error::Internal(e.to_string()))?;
            TypeDecl::Array {
                item: Box::new(ty),
                len,
            }
        }
        Rule::Primitive => {
            let primitive_type =
                PrimitiveType::from_str(p.as_str()).map_err(|e| Error::Internal(e.to_string()))?;
            TypeDecl::Primitive(primitive_type)
        }
        Rule::Named => {
            let mut name = String::new();
            let mut generics: Vec<TypeDecl> = Vec::new();
            for part in p.into_inner() {
                match part.as_rule() {
                    Rule::Ident => name = part.as_str().to_string(),
                    Rule::Generics => {
                        for t in part.into_inner() {
                            generics.push(parse_type_decl(t)?);
                        }
                    }
                    _ => {}
                }
            }
            TypeDecl::Named { name, generics }
        }
        other => {
            return Err(Error::Rule(RuleError::Unexpected(format!(
                "unexpected rule in TypeDecl: {other:?}"
            ))));
        }
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

fn parse_field(p: Pair<'_, Rule>) -> Result<StructField> {
    let mut it = p.into_inner();
    let (docs, annotations) = parse_docs_and_annotations(&mut it)?;
    let part = it.next().ok_or(Error::Rule(RuleError::Expected(
        "expected Ident | TypeDecl".to_string(),
    )))?;
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
            // TODO: Alias is not implemented
            Err(Error::ValidationError("unimplmented AliasDecl".to_string()))
        }
        _ => Err(Error::Rule(RuleError::Unexpected(
            "expected StructDecl | EnumDecl | AliasDecl".to_string(),
        ))),
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
            _ => {
                return Err(Error::Rule(RuleError::Unexpected(
                    "expected StructDef | TupleDef | UnitDef".to_string(),
                )));
            }
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
            _ => {
                return Err(Error::Rule(RuleError::Unexpected(
                    "expected TypeParams | Variants".to_string(),
                )));
            }
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
            _ => {
                return Err(Error::Rule(RuleError::Unexpected(
                    "expected Fields".to_string(),
                )));
            }
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
    let kind = if annotations.iter().any(|(k, _)| k == "query") {
        FunctionKind::Query
    } else {
        FunctionKind::Command
    };
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
                output = Some(parse_type_decl(part.into_inner().next().ok_or(
                    Error::Rule(RuleError::Expected("expect TypeDecl".to_string())),
                )?)?)
            }
            Rule::Throws => {
                throws = Some(parse_type_decl(part.into_inner().next().ok_or(
                    Error::Rule(RuleError::Expected("expect TypeDecl".to_string())),
                )?)?)
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
        kind,
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
        if part.as_rule() == Rule::Params {
            for p in part.into_inner().filter(|x| x.as_rule() == Rule::Param) {
                params.push(parse_param(p)?);
            }
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
                    .filter(|x| x.as_rule() == Rule::ServiceExpo)
                {
                    let mut sit = s.into_inner();
                    let (docs, annotations) = parse_docs_and_annotations(&mut sit)?;
                    let mut name = expect_next(&mut sit, parse_ident)?;
                    let mut route = None;
                    if let Some(p) = sit.next()
                        && p.as_rule() == Rule::Ident
                    {
                        route = Some(name);
                        name = p.as_str().to_string();
                    }
                    services.push(ServiceExpo {
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
            _ => {
                return Err(Error::Rule(RuleError::Unexpected(
                    "expected ConstructorsBlock | ServicesBlock | TypesBlock".to_string(),
                )));
            }
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

fn parse_docs_and_annotations(pairs: &mut Pairs<Rule>) -> Result<(Vec<String>, Vec<Annotation>)> {
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
                    if d.as_rule() == Rule::StrToEol {
                        docs.push(d.as_str().trim().to_string())
                    }
                }
            }
            Rule::LocalAnn => {
                // pop pair
                let _ = pairs.next();
                let ann = parse_annotation(p)?;
                if ann.0 == "doc" {
                    if let Some(val) = ann.1 {
                        docs.push(val);
                    }
                } else {
                    anns.push(ann);
                }
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
    Err(Error::Rule(RuleError::Expected(
        "expected next Rule".to_string(),
    )))
}

fn expect_rule<'a>(
    it: &mut impl Iterator<Item = Pair<'a, Rule>>,
    r: Rule,
) -> Result<Pair<'a, Rule>> {
    if let Some(p) = it.next() {
        if p.as_rule() == r {
            return Ok(p);
        } else {
            return Err(Error::Rule(RuleError::Expected(format!(
                "expected {:?}, but found {:?}",
                r,
                p.as_rule()
            ))));
        }
    }
    Err(Error::Rule(RuleError::Expected(format!(
        "expected {r:?}"
    ))))
}

// ------------------------------ Tests ----------------------------------------
#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn parse_docs_and_annotations_lines() {
        const SRC: &str = r#"
            // Comment
            /// Defines status of some point as colored by somebody or dead for some reason.
            /// Dead point - won't be available for coloring anymore.
            @indexed
            "#;

        let mut pairs = IdlParser::parse(Rule::DocsAndAnnotations, SRC).expect("parse idl");
        let (docs, anns) = parse_docs_and_annotations(&mut pairs).expect("parse annotations");

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

        let doc = parse_idl(SRC).expect("parse idl");

        assert_eq!(3, doc.globals.len());
    }

    #[test]
    fn parse_vector_of_tuples() {
        use PrimitiveType::*;
        use TypeDecl::*;

        const SRC: &str = r#"[(Point<u32>, Option<PointStatus>, u32)]"#;
        let mut pairs = IdlParser::parse(Rule::TypeDecl, SRC).expect("parse idl");
        let ty = expect_next(&mut pairs, parse_type_decl).expect("parse TypeDecl");

        assert_eq!(
            ty,
            Slice {
                item: Box::new(Tuple {
                    types: vec![
                        Named {
                            name: "Point".to_string(),
                            generics: vec![Primitive(U32)]
                        },
                        TypeDecl::option(TypeDecl::named("PointStatus".to_string())),
                        Primitive(U32)
                    ]
                })
            }
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

        assert_eq!(
            func,
            ServiceFunc {
                name: "ColorPoint".to_string(),
                params: vec![
                    FuncParam {
                        name: "point".to_string(),
                        type_decl: TypeDecl::tuple(vec![Primitive(U32), Primitive(U32)])
                    },
                    FuncParam {
                        name: "color".to_string(),
                        type_decl: TypeDecl::named("Color".to_string())
                    }
                ],
                output: Primitive(Void),
                throws: Some(Named {
                    name: "ColorError".to_string(),
                    generics: vec![]
                }),
                kind: FunctionKind::Command,
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
        const SRC: &str = include_str!("../tests/idls/test.idl");

        let doc = parse_idl(SRC).expect("parse idl");

        assert_eq!(3, doc.globals.len());
        assert_eq!(2, doc.services.len());
    }

    #[test]
    fn parse_demo_idl() {
        const SRC: &str = include_str!("../tests/idls/demo.idl");

        let doc = parse_idl(SRC).expect("parse idl");

        assert_eq!(3, doc.globals.len());
    }

    #[test]
    fn parse_idl_rejects_multiple_programs() {
        const SRC: &str = r#"
            program First {}
            program Second {}
        "#;

        let err = parse_idl(SRC).expect_err("multiple programs should fail");
        assert!(matches!(err, Error::ValidationError(_)));
        assert!(
            err.to_string()
                .contains("Expected at most one program per IDL document")
        );
    }

    #[test]
    fn parse_alias_decl_not_supported() {
        const SRC: &str = r#"alias AliasName = u32;"#;

        let mut pairs = IdlParser::parse(Rule::AliasDecl, SRC).expect("parse alias");
        let err =
            parse_type(pairs.next().expect("alias")).expect_err("alias should not be supported");
        assert!(err.to_string().contains("unimplmented AliasDecl"));
    }
}
