use anyhow::Result;
use convert_case::{Case, Casing};
use handlebars::Handlebars;
use sails_idl_parser_v2::{
    ast::{IdlDoc, PrimitiveType, TypeDecl},
    parse_idl,
};
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
    pub payable: bool,
    pub returns_value: bool,
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
    let doc = parse_idl(idl_content)?;

    let contract_name = name.to_string().to_case(Case::UpperCamel);

    let contract_data = ContractData {
        contract_name: contract_name.clone(),
        pragma_version: consts::PRAGMA_VERSION.to_string(),
        functions: functions_from_idl(&doc)?,
        events: events_from_idl(&doc)?,
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

fn functions_from_idl(doc: &IdlDoc) -> Result<Vec<FunctionData>> {
    let mut functions = Vec::new();

    if let Some(program) = &doc.program {
        for func in &program.ctors {
            let mut args = Vec::new();
            for p in &func.params {
                let arg = ArgData {
                    ty: p.type_decl.get_ty()?,
                    name: p.name.to_case(Case::Camel),
                    mem_location: p.type_decl.get_mem_location(),
                };
                args.push(arg);
            }
            functions.push(FunctionData {
                name: func.name.to_case(Case::Camel),
                reply_type: None, // Constructors don't have replies in this sense
                reply_mem_location: None,
                payable: has_tag(&func.docs, "#[payable]"),
                returns_value: false, // Constructors don't return CommandReply values
                args,
            });
        }
    }

    for svc in &doc.services {
        for f in &svc.funcs {
            let mut args = Vec::new();
            for p in &f.params {
                let arg = ArgData {
                    ty: p.type_decl.get_ty()?,
                    name: p.name.to_case(Case::Camel),
                    mem_location: p.type_decl.get_mem_location(),
                };
                args.push(arg);
            }
            let reply_type = if f.output != TypeDecl::Primitive(PrimitiveType::Void) {
                Some(f.output.get_ty()?)
            } else {
                None
            };
            functions.push(FunctionData {
                name: format!("{}{}", svc.name.name, f.name)
                    .as_str()
                    .to_case(Case::Camel),
                reply_type,
                reply_mem_location: f.output.get_mem_location(),
                payable: has_tag(&f.docs, "#[payable]"),
                returns_value: has_tag(&f.docs, "#[returns_value]"),
                args,
            });
        }
    }

    Ok(functions)
}

fn events_from_idl(doc: &IdlDoc) -> Result<Vec<EventData>> {
    let mut events = Vec::new();

    for svc in &doc.services {
        for e in &svc.events {
            let mut args = Vec::new();
            for f in &e.def.fields {
                let arg = EventArgData {
                    ty: f.type_decl.get_ty()?,
                    indexed: f.docs.iter().any(|doc| doc.contains("#[indexed]")),
                    name: f.name.as_ref().map(|name| name.to_case(Case::Camel)),
                };
                args.push(arg);
            }
            events.push(EventData {
                name: e.name.to_string(),
                args,
            });
        }
    }

    Ok(events)
}

fn has_tag(docs: &[String], tag: &str) -> bool {
    docs.iter().any(|doc| doc.contains(tag))
}
