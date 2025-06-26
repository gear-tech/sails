use anyhow::{Result, bail};
use convert_case::{Case, Casing};
use handlebars::Handlebars;
use sails_idl_parser::ast::{Program, TypeDecl, TypeDef, parse_idl};
use serde::Serialize;
use typedecl_to_sol::TypeDeclToSol;

mod consts;
mod typedecl_to_sol;

#[derive(Serialize)]
struct ArgData {
    pub ty: String,
    pub name: String,
    pub mem_location: Option<String>,
}

#[derive(Serialize)]
struct FunctionData {
    pub name: String,
    pub args: Vec<ArgData>,
    pub reply_type: Option<String>,
    pub reply_mem_location: Option<String>,
}

#[derive(Serialize)]
struct EventArgData {
    pub ty: String,
    pub indexed: bool,
    pub name: Option<String>,
}

#[derive(Serialize)]
struct EventData {
    pub name: String,
    pub args: Vec<EventArgData>,
}

#[derive(Serialize)]
struct ContractData {
    pub pragma_version: String,
    pub contract_name: String,
    pub functions: Vec<FunctionData>,
    pub events: Vec<EventData>,
}

pub struct GenerateContractResult {
    pub data: Vec<u8>,
    pub name: String,
}

pub fn generate_solidity_contract(idl_content: &str, name: &str) -> Result<GenerateContractResult> {
    let program = parse_idl(idl_content)?;

    let contract_name = name.to_string().to_case(Case::UpperCamel);

    let contract_data = ContractData {
        contract_name: contract_name.clone(),
        pragma_version: consts::PRAGMA_VERSION.to_string(),
        functions: functions_from_idl(&program)?,
        events: events_from_idl(&program)?,
    };

    let mut handlebars = Handlebars::new();
    handlebars.register_template_string("contract", consts::CONTRACT_TEMPLATE)?;

    let mut contract = Vec::new();

    handlebars.render_to_write("contract", &contract_data, &mut contract)?;

    Ok(GenerateContractResult {
        data: contract,
        name: contract_name,
    })
}

fn functions_from_idl(program: &Program) -> Result<Vec<FunctionData>> {
    let mut functions = Vec::new();

    if let Some(ctor) = program.ctor() {
        for func in ctor.funcs() {
            let mut args = Vec::new();
            for p in func.params() {
                let arg = ArgData {
                    ty: p.type_decl().get_ty()?,
                    name: p.name().to_case(Case::Camel),
                    mem_location: p.type_decl().get_mem_location(),
                };
                args.push(arg);
            }
            functions.push(FunctionData {
                name: func.name().to_case(Case::Camel),
                reply_type: None,
                reply_mem_location: None,
                args,
            });
        }
    }

    for svc in program.services() {
        for f in svc.funcs() {
            let mut args = Vec::new();
            for p in f.params() {
                let arg = ArgData {
                    ty: p.type_decl().get_ty()?,
                    name: p.name().to_case(Case::Camel),
                    mem_location: p.type_decl().get_mem_location(),
                };
                args.push(arg);
            }
            functions.push(FunctionData {
                name: format!("{}{}", svc.name(), f.name())
                    .as_str()
                    .to_case(Case::Camel),
                reply_type: f.output().get_ty().ok(),
                reply_mem_location: f.output().get_mem_location(),
                args,
            });
        }
    }

    Ok(functions)
}

fn events_from_idl(program: &Program) -> Result<Vec<EventData>> {
    let mut events = Vec::new();

    for svc in program.services() {
        for e in svc.events() {
            let mut args = Vec::new();
            match e.type_decl().unwrap() {
                TypeDecl::Def(TypeDef::Struct(def)) => {
                    for f in def.fields() {
                        let arg = EventArgData {
                            ty: f.type_decl().get_ty()?,
                            indexed: false, // TODO: get this from the IDL
                            name: f.name().map(|name| name.to_case(Case::Camel)),
                        };
                        args.push(arg);
                    }
                }
                _ => bail!("Unsupported type"),
            }
            events.push(EventData {
                name: e.name().to_string(),
                args,
            });
        }
    }

    Ok(events)
}
