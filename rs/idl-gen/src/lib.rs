use askama::Template;
pub use errors::*;
pub use program::*;
use sails_idl_meta::*;
use scale_info::{Variant, form::PortableForm};
use std::{fs, io::Write, path::Path};

mod builder;
mod errors;
mod generic_resolver;
mod type_resolver;

const SAILS_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod program {
    use super::*;
    use sails_idl_meta::ProgramMeta;

    pub fn generate_idl<P: ProgramMeta>(mut idl_writer: impl Write) -> Result<()> {
        let doc = build_program_ast::<P>(Some("ProgramToDo".to_string()))?;
        doc.write_into(&mut idl_writer)?;
        Ok(())
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

    pub fn generate_idl<S: ServiceMeta>(mut idl_writer: impl Write) -> Result<()> {
        let doc = build_service_ast("ServiceToDo", AnyServiceMeta::new::<S>())?;
        doc.write_into(&mut idl_writer)?;
        Ok(())
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

fn build_program_ast<P: ProgramMeta>(name: Option<String>) -> Result<IdlDoc> {
    // let
    let service_builders: Vec<_> = P::services()
        .map(|(name, meta)| builder::ServiceBuilder::new(name, meta))
        .collect();
    let services: Vec<_> = service_builders.into_iter().map(|b| b.build()).collect();
    let program = name.map(|name| builder::ProgramBuilder::new::<P>().build(name));
    let doc = IdlDoc {
        globals: vec![
            ("sails".to_string(), Some(SAILS_VERSION.to_string())),
            // ("author".to_string(), Some(gen_meta_info.author)),
            // ("version".to_string(), Some(gen_meta_info.version.format())),
        ],
        program,
        services,
    };
    Ok(doc)
}

fn build_service_ast(name: &'static str, meta: AnyServiceMeta) -> Result<IdlDoc> {
    let services: Vec<_> = vec![builder::ServiceBuilder::new(name, meta).build()];
    let doc = IdlDoc {
        globals: vec![
            ("sails".to_string(), Some(SAILS_VERSION.to_string())),
            // ("author".to_string(), Some(gen_meta_info.author)),
            // ("version".to_string(), Some(gen_meta_info.version.format())),
        ],
        program: None,
        services,
    };
    Ok(doc)
}
