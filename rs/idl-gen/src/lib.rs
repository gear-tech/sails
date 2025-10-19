pub use errors::*;
use handlebars::{Handlebars, handlebars_helper};
use meta::{ExpandedProgramMeta, ExpandedServiceMeta};
pub use program::*;
use sails_interface_id::compute_ids_from_bytes;
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
    let type_names_vec = program_meta.type_names()?.collect::<Vec<_>>();
    let services = program_meta
        .services()
        .map(build_service_idl_data)
        .collect();
    let program_idl_data = ProgramIdlData {
        type_names: type_names_vec.clone(),
        types: program_meta.types().collect(),
        ctors: program_meta.ctors().collect(),
        services,
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
    handlebars.register_helper("hex16", Box::new(hex16));
    handlebars.register_helper("hex32", Box::new(hex32));
    handlebars.register_helper("hex64", Box::new(hex64));

    handlebars
        .render_to_write("idl", &program_idl_data, idl_writer)
        .map_err(Box::new)?;

    Ok(())
}

fn build_service_idl_data<'a>(service: &'a ExpandedServiceMeta) -> ServiceIdlData<'a> {
    let (interface_id32, interface_uid64) = compute_ids_from_bytes(service.canonical_bytes());

    let extends = service
        .extends()
        .iter()
        .map(|ext| ExtendsIdlData {
            name: ext.name.as_str(),
            interface_id32: ext.interface_id32,
            interface_uid64: ext.interface_uid64,
        })
        .collect();

    let commands = service
        .commands()
        .map(
            |(name, params, result_type_id, entry_id, docs)| FuncIdlData {
                name,
                params,
                result_type_id,
                entry_id,
                docs,
            },
        )
        .collect();

    let queries = service
        .queries()
        .map(
            |(name, params, result_type_id, entry_id, docs)| FuncIdlData {
                name,
                params,
                result_type_id,
                entry_id,
                docs,
            },
        )
        .collect();

    let events = service
        .events()
        .map(|(variant, entry_id)| EventIdlData { variant, entry_id })
        .collect();

    ServiceIdlData {
        name: service.name(),
        interface_id32,
        interface_uid64,
        extends,
        commands,
        queries,
        events,
    }
}

type CtorIdlData<'a> = (&'a str, &'a Vec<Field<PortableForm>>, &'a Vec<String>);

#[derive(Serialize)]
struct FuncIdlData<'a> {
    name: &'a str,
    params: &'a Vec<Field<PortableForm>>,
    result_type_id: u32,
    entry_id: u16,
    docs: &'a Vec<String>,
}

#[derive(Serialize)]
struct EventIdlData<'a> {
    variant: &'a Variant<PortableForm>,
    entry_id: u16,
}

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
    interface_id32: u32,
    interface_uid64: u64,
    extends: Vec<ExtendsIdlData<'a>>,
    commands: Vec<FuncIdlData<'a>>,
    queries: Vec<FuncIdlData<'a>>,
    events: Vec<EventIdlData<'a>>,
}

#[derive(Serialize)]
struct ExtendsIdlData<'a> {
    name: &'a str,
    interface_id32: u32,
    interface_uid64: u64,
}

handlebars_helper!(deref: |v: String| { v });
handlebars_helper!(hex16: |v: u16| { format!("{:#06x}", v) });
handlebars_helper!(hex32: |v: u32| { format!("{:#010x}", v) });
handlebars_helper!(hex64: |v: u64| { format!("{:#018x}", v) });
