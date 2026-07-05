use alloc::{string::String, vec::Vec};

use crate::{
    error::Result,
    sol_conversion::{ConversionError, TypeDeclExt},
};
use askama::Template;
use convert_case::{Case, Casing};
use sails_idl_parser_v2::{
    ast::{IdlDoc, PrimitiveType, Type, TypeDecl, codec::has_ethabi_codec},
    parse_idl,
};

struct Arg {
    ty: String,
    name: String,
    mem_location: Option<String>,
}

struct Function {
    name: String,
    args: Vec<Arg>,
    reply_type: Option<String>,
    reply_mem_location: Option<String>,
    payable: bool,
    returns_value: bool,
}

struct EventArg {
    ty: String,
    indexed: bool,
    name: Option<String>,
}

struct Event {
    name: String,
    args: Vec<EventArg>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolidityFile {
    SingleFile,
    InterfaceFile,
    AbiInterfaceFile,
    CallbacksInterfaceFile,
    CallerFile,
}

struct ContractData {
    license_identifier: String,
    solidity_version: String,
    contract_name: String,
    functions: Vec<Function>,
    events: Vec<Event>,
}

macro_rules! define_template {
    ($name:ident, $path:literal) => {
        #[derive(Template)]
        #[template(path = $path)]
        #[allow(dead_code)]
        struct $name {
            license_identifier: String,
            solidity_version: String,
            contract_name: String,
            functions: Vec<Function>,
            events: Vec<Event>,
        }

        impl From<ContractData> for $name {
            fn from(data: ContractData) -> Self {
                Self {
                    license_identifier: data.license_identifier,
                    solidity_version: data.solidity_version,
                    contract_name: data.contract_name,
                    functions: data.functions,
                    events: data.events,
                }
            }
        }
    };
}

define_template!(SingleFile, "single_file.askama");
define_template!(InterfaceFile, "interface_file.askama");
define_template!(AbiInterfaceFile, "abi_interface_file.askama");
define_template!(CallbacksInterfaceFile, "callbacks_interface_file.askama");
define_template!(CallerFile, "caller_file.askama");

pub const LICENSE_IDENTIFIER: &str = "MIT";
pub const SOLIDITY_VERSION: &str = "0.8.35";

pub fn generate_solidity_contract(
    contract_name: &str,
    idl_content: &str,
    solidity_file: SolidityFile,
) -> Result<Vec<u8>> {
    let idl_doc = parse_idl(idl_content)?;

    let contract_data = ContractData {
        license_identifier: LICENSE_IDENTIFIER.into(),
        solidity_version: SOLIDITY_VERSION.into(),
        contract_name: contract_name.into(),
        functions: functions_from_idl(&idl_doc)?,
        events: events_from_idl(&idl_doc)?,
    };

    let rendered = match solidity_file {
        SolidityFile::SingleFile => SingleFile::from(contract_data).render()?,
        SolidityFile::InterfaceFile => InterfaceFile::from(contract_data).render()?,
        SolidityFile::AbiInterfaceFile => AbiInterfaceFile::from(contract_data).render()?,
        SolidityFile::CallbacksInterfaceFile => {
            CallbacksInterfaceFile::from(contract_data).render()?
        }
        SolidityFile::CallerFile => CallerFile::from(contract_data).render()?,
    };

    Ok(rendered.into_bytes())
}

fn resolve_type_decl(decl: &TypeDecl, types: &[Type]) -> Result<String, ConversionError> {
    match decl {
        TypeDecl::Named { name, .. } => types
            .iter()
            .find(|ty| ty.name == *name)
            .and_then(|ty| ty.annotations.iter().find(|(key, _)| key == "sol_type"))
            .and_then(|(_, value)| value.clone())
            .ok_or(ConversionError::UnsupportedType),
        TypeDecl::Array { item, len } => {
            let ty = resolve_type_decl(item, types)?;
            Ok(format!("{ty}[{len}]"))
        }
        TypeDecl::Slice { item } => {
            let ty = resolve_type_decl(item, types)?;
            Ok(format!("{ty}[]"))
        }
        _ => decl.get_ty(),
    }
}

fn functions_from_idl(idl_doc: &IdlDoc) -> Result<Vec<Function>> {
    let mut functions = vec![];

    if let Some(program) = &idl_doc.program {
        for ctor_func in &program.ctors {
            let mut args = vec![];

            for func_param in &ctor_func.params {
                args.push(Arg {
                    ty: resolve_type_decl(&func_param.type_decl, &program.types)?,
                    name: func_param.name.to_case(Case::Camel),
                    mem_location: func_param.type_decl.get_mem_location(),
                });
            }

            functions.push(Function {
                name: ctor_func.name.to_case(Case::Camel),
                reply_type: None, // Constructors don't have replies in this sense
                reply_mem_location: None,
                payable: ctor_func
                    .annotations
                    .iter()
                    .any(|(key, _)| key == "payable"),
                returns_value: false, // Constructors don't return CommandReply values
                args,
            });
        }
    }

    for service_unit in &idl_doc.services {
        for service_func in &service_unit.funcs {
            if !has_ethabi_codec(&service_func.annotations) {
                continue;
            }

            let mut args = vec![];

            for func_param in &service_func.params {
                args.push(Arg {
                    ty: resolve_type_decl(&func_param.type_decl, &service_unit.types)?,
                    name: func_param.name.to_case(Case::Camel),
                    mem_location: func_param.type_decl.get_mem_location(),
                });
            }

            let reply_type = match &service_func.output {
                TypeDecl::Primitive(PrimitiveType::Void) => None,
                output => Some(resolve_type_decl(output, &service_unit.types)?),
            };

            let service_name = &service_unit.name.name;
            let service_func_name = &service_func.name;

            functions.push(Function {
                name: format!("{service_name}{service_func_name}").to_case(Case::Camel),
                reply_type,
                reply_mem_location: service_func.output.get_mem_location(),
                payable: service_func
                    .annotations
                    .iter()
                    .any(|(key, _)| key == "payable"),
                returns_value: service_func
                    .annotations
                    .iter()
                    .any(|(key, _)| key == "returns_value"),
                args,
            });
        }
    }

    Ok(functions)
}

fn events_from_idl(idl_doc: &IdlDoc) -> Result<Vec<Event>> {
    let mut events = vec![];

    for service_unit in &idl_doc.services {
        for enum_variant in &service_unit.events {
            if !has_ethabi_codec(&enum_variant.annotations) {
                continue;
            }

            let mut args = vec![];

            for struct_field in &enum_variant.def.fields {
                args.push(EventArg {
                    ty: resolve_type_decl(&struct_field.type_decl, &service_unit.types)?,
                    indexed: struct_field
                        .annotations
                        .iter()
                        .any(|(key, _)| key == "indexed"),
                    name: struct_field
                        .name
                        .as_ref()
                        .map(|name| name.to_case(Case::Camel)),
                });
            }

            events.push(Event {
                name: enum_variant.name.clone(),
                args,
            });
        }
    }

    Ok(events)
}
