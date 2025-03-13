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

const IDL_TEMPLATE: &str = include_str!("../hbs/idl.hbs");
const COMPOSITE_TEMPLATE: &str = include_str!("../hbs/composite.hbs");
const VARIANT_TEMPLATE: &str = include_str!("../hbs/variant.hbs");

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
        if let Ok(idl_old_content) = fs::read(&path) {
            if idl_new_content == idl_old_content {
                return Ok(());
            }
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
        if let Ok(idl_old_content) = fs::read(&path) {
            if idl_new_content == idl_old_content {
                return Ok(());
            }
        }
        Ok(fs::write(&path, idl_new_content)?)
    }
}

fn render_idl(program_meta: &ExpandedProgramMeta, idl_writer: impl Write) -> Result<()> {
    let program_idl_data = ProgramIdlData {
        type_names: program_meta.type_names()?.collect(),
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

handlebars_helper!(deref: |v: String| { v });
