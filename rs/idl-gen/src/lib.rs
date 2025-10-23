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

// todo [sab] generics?
// todo [sab] add global annotations

// todo [sab] discuss extends section
// (no need to merge fns, or merge but with stating source service -> benefits when same method names corner case)

// todo [sab] which sections can be absent -> adjust template with ifs and add proper indentations

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
        .register_template_string("idlv2", IDLV2_TEMPLATE)
        .map_err(Box::new)?;
    handlebars
        .register_template_string("service", SERVICE_TEMPLATE)
        .map_err(Box::new)?;
    handlebars
        .register_template_string("program", PROGRAM_TEMPLATE)
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
struct Idl2Data {
    #[serde(rename = "program")]
    program_section: ProgramIdlSection,
    services: Vec<ServiceSection>,
}

#[derive(Debug, Serialize)]
struct ProgramIdlSection {
    name: String,
    type_names: Vec<String>,
    ctors: Vec<FunctionIdl>,
    types: Vec<PortableType>,
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
    name: String,
    type_names: Vec<String>,
    extends: Vec<String>,
    events: Vec<Variant<PortableForm>>,
    types: Vec<PortableType>,
    functions: FunctionsSection,
}

#[derive(Debug, Serialize)]
struct FunctionsSection {
    commands: Vec<FunctionIdl>,
    queries: Vec<FunctionIdl>,
}

#[derive(Serialize)]
struct ServiceIdlData<'a> {
    name: &'a str,
    commands: Vec<FuncIdlData<'a>>,
    queries: Vec<FuncIdlData<'a>>,
    events: Vec<&'a Variant<PortableForm>>,
}

handlebars_helper!(deref: |v: String| { v });

#[cfg(test)]
mod tests {
    use crate::meta::ExpandedProgramMeta2;

    use super::*;

    // todo [sab] make proper spaces between sections in service idl and test proper indentations when some sections are missing
    // expected output tests are in tests folder of the crate

    #[test]
    fn test_new_json() {
        use demo::DemoProgram;
        use sails_idl_meta::ProgramMeta;
        let mut source: Vec<u8> = Vec::new();

        let data = ExpandedProgramMeta2::new(
            "Demo".to_string(),
            DemoProgram::constructors(),
            DemoProgram::services(),
        )
        .unwrap();

        let json = serde_json::to_string_pretty(&data).unwrap();
        println!("{}", json);

        let mut hbs = Handlebars::new();
        let _ = hbs.register_template_string("idlv2", IDLV2_TEMPLATE);
        let _ = hbs.register_template_string("service", SERVICE_TEMPLATE);
        let _ = hbs.register_template_string("program", PROGRAM_TEMPLATE);
        hbs.register_helper("deref", Box::new(deref));

        hbs.render_to_write("idlv2", &data, &mut source).unwrap();
        println!("{}", String::from_utf8_lossy(&source));
    }

    /// Test IDL generation with user-defined types in program section
    /// (constructors with custom types as arguments)
    #[test]
    fn test_program_with_custom_types() {
        use scale_info::{MetaType, TypeInfo};

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct CustomType {
            pub value: u32,
            pub name: String,
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct AnotherCustomType {
            pub id: u64,
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum ProgramConstructors {
            Default(DefaultParams),
            WithCustomType(WithCustomTypeParams),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct DefaultParams {}

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct WithCustomTypeParams {
            pub custom: CustomType,
            pub another: AnotherCustomType,
        }

        let mut source: Vec<u8> = Vec::new();

        let data = ExpandedProgramMeta2::new(
            "ProgramWithCustomTypes".to_string(),
            MetaType::new::<ProgramConstructors>(),
            std::iter::empty(),
        )
        .unwrap();

        let json = serde_json::to_string_pretty(&data).unwrap();
        println!("{}", json);

        let mut hbs = Handlebars::new();
        let _ = hbs.register_template_string("idlv2", IDLV2_TEMPLATE);
        let _ = hbs.register_template_string("service", SERVICE_TEMPLATE);
        let _ = hbs.register_template_string("program", PROGRAM_TEMPLATE);
        hbs.register_helper("deref", Box::new(deref));

        hbs.render_to_write("idlv2", &data, &mut source).unwrap();
        println!("{}", String::from_utf8_lossy(&source));
    }

    /// Test IDL generation with unit structs in types section
    #[test]
    fn test_unit_struct_types() {
        use scale_info::{MetaType, TypeInfo};

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct UnitStruct;

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct AnotherUnitStruct;

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct TupleStruct(u32, String);

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct RegularStruct {
            pub field1: u32,
            pub field2: String,
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum ProgramConstructors {
            Default(DefaultParams),
            WithUnitStructs(WithUnitStructsParams),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct DefaultParams {}

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct WithUnitStructsParams {
            pub unit: UnitStruct,
            pub another_unit: AnotherUnitStruct,
            pub tuple: TupleStruct,
            pub regular: RegularStruct,
        }

        // Service types
        use sails_idl_meta::{AnyServiceMeta, ServiceMeta};

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum NoEvents {}

        struct StructService;

        impl ServiceMeta for StructService {
            type CommandsMeta = ServiceCommands;
            type QueriesMeta = ServiceQueries;
            type EventsMeta = NoEvents;
            const BASE_SERVICES: &'static [sails_idl_meta::AnyServiceMetaFn] = &[];
            const ASYNC: bool = false;
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum ServiceCommands {
            ProcessUnit(ProcessUnitParams, ProcessUnitOutput),
            CreateStruct(CreateStructParams, CreateStructOutput),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        enum ServiceQueries {
            GetAllStructs(GetAllStructsParams, GetAllStructsOutput),
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct ProcessUnitParams {
            pub unit: UnitStruct,
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct ProcessUnitOutput(bool);

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct CreateStructParams {
            pub regular: RegularStruct,
            pub tuple: TupleStruct,
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct CreateStructOutput {
            pub id: u64,
            pub unit: UnitStruct,
        }

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct GetAllStructsParams {}

        #[derive(TypeInfo)]
        #[allow(unused)]
        struct GetAllStructsOutput {
            pub units: Vec<UnitStruct>,
            pub regulars: Vec<RegularStruct>,
            pub tuples: Vec<TupleStruct>,
        }

        let mut source: Vec<u8> = Vec::new();

        let data = ExpandedProgramMeta2::new(
            "UnitStructProgram".to_string(),
            MetaType::new::<ProgramConstructors>(),
            vec![("StructService", AnyServiceMeta::new::<StructService>())].into_iter(),
        )
        .unwrap();

        let json = serde_json::to_string_pretty(&data).unwrap();
        println!("{}", json);

        let mut hbs = Handlebars::new();
        let _ = hbs.register_template_string("idlv2", IDLV2_TEMPLATE);
        let _ = hbs.register_template_string("service", SERVICE_TEMPLATE);
        let _ = hbs.register_template_string("program", PROGRAM_TEMPLATE);
        hbs.register_helper("deref", Box::new(deref));

        hbs.render_to_write("idlv2", &data, &mut source).unwrap();
        println!("{}", String::from_utf8_lossy(&source));
    }
}
