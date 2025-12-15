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

    pub fn generate_idl<P: ProgramMeta>(
        program_name: Option<&str>,
        mut idl_writer: impl Write,
    ) -> Result<()> {
        let doc = build_program_ast::<P>(program_name)?;
        doc.write_into(&mut idl_writer)?;
        Ok(())
    }

    pub fn generate_idl_to_file<P: ProgramMeta>(
        program_name: Option<&str>,
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

pub mod service {
    use super::*;
    use sails_idl_meta::{AnyServiceMeta, ServiceMeta};

    pub fn generate_idl<S: ServiceMeta>(
        service_name: &str,
        mut idl_writer: impl Write,
    ) -> Result<()> {
        let doc = build_service_ast(service_name, AnyServiceMeta::new::<S>())?;
        doc.write_into(&mut idl_writer)?;
        Ok(())
    }

    pub fn generate_idl_to_file<S: ServiceMeta>(path: impl AsRef<Path>) -> Result<()> {
        let service_name = path
            .as_ref()
            .file_name()
            .map(|f| f.to_string_lossy().to_string())
            .unwrap_or_else(|| "Service".to_string());

        let mut idl_new_content = Vec::new();
        generate_idl::<S>(service_name.as_str(), &mut idl_new_content)?;
        if let Ok(idl_old_content) = fs::read(&path)
            && idl_new_content == idl_old_content
        {
            return Ok(());
        }
        Ok(fs::write(&path, idl_new_content)?)
    }
}

fn build_program_ast<P: ProgramMeta>(program_name: Option<&str>) -> Result<IdlDoc> {
    let mut services = Vec::new();
    for (name, meta) in P::services() {
        services.extend(builder::ServiceBuilder::new(name, &meta).build()?);
    }
    let program = if let Some(name) = program_name {
        Some(builder::ProgramBuilder::new::<P>().build(name.to_string())?)
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
    let services = builder::ServiceBuilder::new(service_name, &meta).build()?;
    let doc = IdlDoc {
        globals: vec![("sails".to_string(), Some(SAILS_VERSION.to_string()))],
        program: None,
        services,
    };
    Ok(doc)
}
