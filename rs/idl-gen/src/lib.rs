use crate::type_names::{FinalizedName, FinalizedRawName};
pub use errors::*;
use handlebars::{Handlebars, handlebars_helper};
use meta::ExpandedProgramMeta;
pub use program::*;
use scale_info::{Field, PortableType, Variant, form::PortableForm};
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::{fmt::Display, fs, io::Write, path::Path};

mod errors;
mod meta;
mod meta2;
mod type_names;

const IDL_GEN_VERSION: &str = "2.0.0";

const IDL_TEMPLATE: &str = include_str!("../hbs/idl.hbs");
const COMPOSITE_TEMPLATE: &str = include_str!("../hbs/composite.hbs");
const VARIANT_TEMPLATE: &str = include_str!("../hbs/variant.hbs");

const IDLV2_TEMPLATE: &str = include_str!("../hbs/idlv2.hbs");
const SERVICE_TEMPLATE: &str = include_str!("../hbs/service.hbs");
const PROGRAM_TEMPLATE: &str = include_str!("../hbs/program.hbs");

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
        .register_template_string("idl", IDL_TEMPLATE)
        .map_err(Box::new)?;
    handlebars
        .register_template_string("composite", COMPOSITE_TEMPLATE)
        .map_err(Box::new)?;
    handlebars
        .register_template_string("variant", VARIANT_TEMPLATE)
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
struct ServiceIdlData<'a> {
    name: &'a str,
    commands: Vec<FuncIdlData<'a>>,
    queries: Vec<FuncIdlData<'a>>,
    events: Vec<&'a Variant<PortableForm>>,
}

pub mod program2 {
    use super::*;
    use sails_idl_meta::ProgramMeta;

    pub fn generate_idl<P: ProgramMeta>(
        program_name: String,
        idl_writer: impl Write,
    ) -> Result<()> {
        render_idlv2(
            meta2::ExpandedProgramMeta::new(
                Some((program_name, P::constructors())),
                P::services(),
            )?,
            idl_writer,
        )
    }

    pub fn generate_idl_to_file<P: ProgramMeta>(
        program_name: String,
        path: impl AsRef<Path>,
    ) -> Result<()> {
        let mut idl_new_content = Vec::new();
        generate_idl::<P>(program_name, &mut idl_new_content)?;
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

pub mod service2 {
    use super::*;
    use sails_idl_meta::{AnyServiceMeta, ServiceMeta};

    pub fn generate_idl<S: ServiceMeta>(idl_writer: impl Write) -> Result<()> {
        render_idlv2(
            meta2::ExpandedProgramMeta::new(
                None,
                vec![("", AnyServiceMeta::new::<S>())].into_iter(),
            )?,
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

fn render_idlv2(program_meta: meta2::ExpandedProgramMeta, idl_writer: impl Write) -> Result<()> {
    let idl_data = IdlData {
        program_section: program_meta.program,
        services: program_meta.services,
        sails_version: IDL_GEN_VERSION.to_string(),
    };

    let mut handlebars = Handlebars::new();
    handlebars
        .register_template_string("idlv2", IDLV2_TEMPLATE)
        .map_err(Box::new)?;
    handlebars
        .register_partial("program", PROGRAM_TEMPLATE)
        .map_err(Box::new)?;
    handlebars
        .register_partial("service", SERVICE_TEMPLATE)
        .map_err(Box::new)?;
    handlebars.register_helper("deref", Box::new(deref));
    handlebars.register_helper("any_field_has_docs", Box::new(any_field_has_docs));
    handlebars.register_helper("has_functions", Box::new(has_functions));
    handlebars.register_helper("has_key", Box::new(has_key));

    handlebars
        .render_to_write("idlv2", &idl_data, idl_writer)
        .map_err(Box::new)?;

    Ok(())
}

#[derive(Serialize)]
struct IdlData {
    #[serde(rename = "program", skip_serializing_if = "Option::is_none")]
    program_section: Option<ProgramIdlSection>,
    services: Vec<ServiceSection>,
    sails_version: String,
}

#[derive(Debug, Serialize)]
struct ProgramIdlSection {
    name: String,
    concrete_names: Vec<FinalizedName>,
    ctors: Vec<FunctionIdl>,
    types: Vec<FinalizedRawName>,
    services: Vec<String>,
}

#[derive(Debug, Serialize)]
struct FunctionIdl {
    name: String,
    args: Vec<FunctionArgumentIdl>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result_ty: Option<FunctionResultIdl>,
    docs: Vec<String>,
}

#[derive(Debug, Serialize)]
struct FunctionResultIdl {
    // The field is optional, because `()` value is treated as no-result,
    // so has `None` value
    #[serde(skip_serializing_if = "Option::is_none")]
    res: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    err: Option<u32>,
}

#[derive(Debug, Serialize)]
struct FunctionArgumentIdl {
    name: String,
    #[serde(rename = "type")]
    ty: u32,
}

#[derive(Debug, Serialize)]
struct ServiceSection {
    name: ServiceNameTy,
    concrete_names: Vec<FinalizedName>,
    extends: Vec<String>,
    events: Vec<Variant<PortableForm>>,
    types: Vec<FinalizedRawName>,
    functions: FunctionsSection,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum ServiceNameTy {
    Base(String),
    Main(String),
}

impl ServiceNameTy {
    #[cfg(test)]
    pub fn as_str(&self) -> &str {
        match self {
            ServiceNameTy::Base(name) => name,
            ServiceNameTy::Main(name) => name,
        }
    }

    pub fn main(&self) -> Option<String> {
        match self {
            ServiceNameTy::Base(_) => None,
            ServiceNameTy::Main(name) => Some(name.clone()),
        }
    }
}

impl Display for ServiceNameTy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServiceNameTy::Base(name) => write!(f, "{name}",),
            ServiceNameTy::Main(name) => write!(f, "{name}",),
        }
    }
}

#[derive(Debug, Serialize)]
struct FunctionsSection {
    commands: Vec<FunctionIdl>,
    queries: Vec<FunctionIdl>,
}

handlebars_helper!(deref: |v: String| { v });
handlebars_helper!(any_field_has_docs: |fields: JsonValue| {
    fields.as_array().is_some_and(|arr| {
        arr.iter().any(|f| {
            f.get("docs")
                .and_then(|d| d.as_array())
                .is_some_and(|docs| !docs.is_empty())
        })
    })
});
handlebars_helper!(has_functions: |functions: JsonValue| {
    functions.as_object().is_some_and(|obj| {
        let has_commands = obj.get("commands")
            .and_then(|c| c.as_array())
            .is_some_and(|arr| !arr.is_empty());

        let has_queries = obj.get("queries")
            .and_then(|q| q.as_array())
            .is_some_and(|arr| !arr.is_empty());

        has_commands || has_queries
    })
});
handlebars_helper!(has_key: |obj: JsonValue, key: JsonValue| {
    let JsonValue::String(key) = key else {
        panic!("key must be string")
    };

    obj.as_object()
        .and_then(|o| o.get(&key))
        .is_some()
});
