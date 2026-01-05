#![no_std]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use alloc::{
    boxed::Box,
    collections::{BTreeMap, BTreeSet},
    format,
    string::{String, ToString as _},
    vec,
    vec::Vec,
};
use askama::Template;
pub use errors::*;
pub use program::*;
use sails_idl_meta::*;
use scale_info::{Variant, form::PortableForm};

mod builder;
mod errors;
mod generic_resolver;
mod type_resolver;

const SAILS_VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod program {
    use super::*;
    use sails_idl_meta::ProgramMeta;

    pub fn generate_idl<P: ProgramMeta>(
        program_name: Option<&str>,
        mut idl_writer: impl core::fmt::Write,
    ) -> Result<()> {
        let doc = build_program_ast::<P>(program_name)?;
        doc.render_into(&mut idl_writer)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    pub fn generate_idl_to_file<P: ProgramMeta>(
        program_name: Option<&str>,
        path: impl AsRef<std::path::Path>,
    ) -> Result<()> {
        let mut idl_new_content = String::new();
        generate_idl::<P>(program_name, &mut idl_new_content)?;
        if let Ok(idl_old_content) = std::fs::read_to_string(&path)
            && idl_new_content == idl_old_content
        {
            return Ok(());
        }
        if let Some(dir_path) = path.as_ref().parent() {
            std::fs::create_dir_all(dir_path)?;
        }
        Ok(std::fs::write(&path, idl_new_content)?)
    }
}

pub mod service {
    use super::*;
    use sails_idl_meta::{AnyServiceMeta, ServiceMeta};

    pub fn generate_idl<S: ServiceMeta>(
        service_name: &str,
        mut idl_writer: impl core::fmt::Write,
    ) -> Result<()> {
        let doc = build_service_ast(service_name, AnyServiceMeta::new::<S>())?;
        doc.render_into(&mut idl_writer)?;
        Ok(())
    }

    #[cfg(feature = "std")]
    pub fn generate_idl_to_file<S: ServiceMeta>(
        service_name: &str,
        path: impl AsRef<std::path::Path>,
    ) -> Result<()> {
        let mut idl_new_content = String::new();
        generate_idl::<S>(service_name, &mut idl_new_content)?;
        if let Ok(idl_old_content) = std::fs::read_to_string(&path)
            && idl_new_content == idl_old_content
        {
            return Ok(());
        }
        if let Some(dir_path) = path.as_ref().parent() {
            std::fs::create_dir_all(dir_path)?;
        }
        Ok(std::fs::write(&path, idl_new_content)?)
    }
}

fn build_program_ast<P: ProgramMeta>(program_name: Option<&str>) -> Result<IdlDoc> {
    let mut services = Vec::new();
    for (name, meta) in P::services() {
        builder::ServiceBuilder::new(name, &meta).build(&mut services)?;
    }
    let program = if let Some(name) = program_name {
        Some(builder::ProgramBuilder::new::<P>().build(name.to_string(), &services)?)
    } else {
        None
    };
    let doc = IdlDoc {
        globals: vec![("sails".to_string(), Some(SAILS_VERSION.to_string()))],
        program,
        services,
    };
    Ok(doc)
}

fn build_service_ast(service_name: &str, meta: AnyServiceMeta) -> Result<IdlDoc> {
    let mut services = Vec::new();
    builder::ServiceBuilder::new(service_name, &meta).build(&mut services)?;
    let doc = IdlDoc {
        globals: vec![("sails".to_string(), Some(SAILS_VERSION.to_string()))],
        program: None,
        services,
    };
    Ok(doc)
}
