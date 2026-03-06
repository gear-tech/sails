use anyhow::{Context, Result, anyhow, bail};
use convert_case::{Case, Casing};
use genco::prelude::*;
use std::{collections::HashSet, fs, io::Write, path::Path};

pub struct SolPath<'a>(&'a Path);
pub struct SolString<'a>(&'a str);

pub struct ClientGenerator<'a, S> {
    contract_name: Option<&'a str>,
    with_no_std: bool,
    sol: S,
}

impl<'a, S> ClientGenerator<'a, S> {
    pub fn with_contract_name(self, contract_name: &'a str) -> Self {
        Self {
            contract_name: Some(contract_name),
            ..self
        }
    }

    pub fn with_no_std(self, with_no_std: bool) -> Self {
        Self {
            with_no_std,
            ..self
        }
    }
}

impl<'a> ClientGenerator<'a, SolPath<'a>> {
    pub fn from_sol_path(sol_path: &'a Path) -> Self {
        Self {
            contract_name: None,
            with_no_std: false,
            sol: SolPath(sol_path),
        }
    }

    pub fn generate(self) -> Result<String> {
        let source = fs::read_to_string(self.sol.0)
            .with_context(|| format!("Failed to open {} for reading", self.sol.0.display()))?;
        self.with_sol(&source).generate()
    }

    pub fn generate_to(self, out_path: impl AsRef<Path>) -> Result<()> {
        let out_path = out_path.as_ref();
        let code = self.generate().context("failed to generate Solidity client")?;
        fs::write(out_path, code)
            .with_context(|| format!("Failed to write generated client to {}", out_path.display()))
    }

    fn with_sol(self, sol: &'a str) -> ClientGenerator<'a, SolString<'a>> {
        ClientGenerator {
            contract_name: self.contract_name,
            with_no_std: self.with_no_std,
            sol: SolString(sol),
        }
    }
}

impl<'a> ClientGenerator<'a, SolString<'a>> {
    pub fn from_sol(sol: &'a str) -> Self {
        Self {
            contract_name: None,
            with_no_std: false,
            sol: SolString(sol),
        }
    }

    pub fn generate(self) -> Result<String> {
        let spec = parse_contract(self.sol.0, self.contract_name)?;
        let code = generate_client_code(&spec, self.with_no_std);
        Ok(pretty_with_rustfmt(&code))
    }

    pub fn generate_to(self, out_path: impl AsRef<Path>) -> Result<()> {
        let out_path = out_path.as_ref();
        let code = self.generate().context("failed to generate Solidity client")?;
        fs::write(out_path, code)
            .with_context(|| format!("Failed to write generated client to {}", out_path.display()))
    }
}

#[derive(Debug)]
struct ContractSpec {
    rust_name: String,
    has_constructor: bool,
    functions: Vec<FunctionSpec>,
}

#[derive(Debug)]
struct FunctionSpec {
    signature: String,
    method_name: String,
    io_name: String,
    signature_const: String,
}

fn parse_contract(source: &str, wanted_contract_name: Option<&str>) -> Result<ContractSpec> {
    let token_stream: proc_macro2::TokenStream = source
        .parse()
        .map_err(|e| anyhow!("Failed to tokenize Solidity source: {e}"))?;
    let file = syn_solidity::parse2(token_stream).context("Failed to parse Solidity source")?;

    let contracts: Vec<&syn_solidity::ItemContract> = file
        .items
        .iter()
        .filter_map(|item| match item {
            syn_solidity::Item::Contract(contract) => Some(contract),
            _ => None,
        })
        .collect();

    if contracts.is_empty() {
        bail!("No contract/interface/library declarations found in Solidity source");
    }

    let selected = if let Some(name) = wanted_contract_name {
        contracts
            .iter()
            .find(|c| c.name.to_string() == name)
            .copied()
            .ok_or_else(|| {
                let names = contracts
                    .iter()
                    .map(|c| c.name.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                anyhow!("Contract `{name}` not found. Available: {names}")
            })?
    } else if contracts.len() == 1 {
        contracts[0]
    } else {
        let names = contracts
            .iter()
            .map(|c| c.name.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        bail!("Multiple contracts/interfaces found. Please set `contract_name`. Available: {names}");
    };

    let mut has_constructor = false;
    let mut seen_names = HashSet::new();
    let mut functions = Vec::new();

    for item in &selected.body {
        let syn_solidity::Item::Function(function) = item else {
            continue;
        };

        match &function.kind {
            syn_solidity::FunctionKind::Constructor(_) => {
                if has_constructor {
                    bail!(
                        "Multiple constructors found in `{}` declaration; only one constructor is supported",
                        selected.name
                    );
                }
                has_constructor = true;
            }
            syn_solidity::FunctionKind::Function(_) => {
                let Some(name) = function.name.as_ref() else {
                    continue;
                };
                let name = name.to_string();

                if !seen_names.insert(name.clone()) {
                    bail!(
                        "Function overloading is not supported yet in generated client: `{name}` appears multiple times"
                    );
                }

                let signature = format!(
                    "{}({})",
                    name,
                    function.parameters.type_strings().collect::<Vec<_>>().join(",")
                );
                let method_name = name.to_case(Case::Snake);
                let io_name = name.to_case(Case::Pascal);
                let signature_const = format!("{}_SIGNATURE", method_name.to_case(Case::UpperSnake));

                functions.push(FunctionSpec {
                    signature,
                    method_name,
                    io_name,
                    signature_const,
                });
            }
            _ => {}
        }
    }

    if functions.is_empty() && !has_constructor {
        bail!(
            "No callable `function` or `constructor` items found in `{}` declaration",
            selected.name
        );
    }

    Ok(ContractSpec {
        rust_name: selected.name.to_string().to_case(Case::Pascal),
        has_constructor,
        functions,
    })
}

fn generate_client_code(spec: &ContractSpec, with_no_std: bool) -> String {
    let mut tokens = rust::Tokens::new();
    let client_name_owned = format!("{}Client", spec.rust_name);
    let client_name = client_name_owned.as_str();

    quote_in! { tokens =>
        use sails_rs::{
            ActorId,
            client::{
                GearEnv,
                Identifiable,
                MethodMeta,
                PendingCall,
                PendingCtor,
                ServiceCall,
            },
            solidity,
        };

        #[derive(Debug, Clone)]
        pub struct $client_name<E: GearEnv> {
            env: E,
            program_id: ActorId,
        }

        impl<E: GearEnv> $client_name<E> {
            pub fn new(env: E, program_id: ActorId) -> Self {
                Self { env, program_id }
            }

            pub fn with_program_id(mut self, program_id: ActorId) -> Self {
                self.program_id = program_id;
                self
            }

            pub fn program_id(&self) -> ActorId {
                self.program_id
            }

            pub fn env(&self) -> &E {
                &self.env
            }
        }
    };

    if spec.has_constructor {
        quote_in! { tokens =>
            pub struct Constructor;

            impl Identifiable for Constructor {
                const INTERFACE_ID: sails_rs::InterfaceId = sails_rs::InterfaceId::zero();
            }

            impl MethodMeta for Constructor {
                const ENTRY_ID: u16 = u16::MAX;
            }

            impl ServiceCall for Constructor {
                type Params = sails_rs::prelude::Vec<u8>;
                type Reply = ();

                fn encode_params_with_header(
                    _route_idx: u8,
                    value: &Self::Params,
                ) -> sails_rs::prelude::Vec<u8> {
                    value.clone()
                }

                fn decode_reply_with_header(
                    _route_idx: u8,
                    _payload: impl AsRef<[u8]>,
                ) -> Result<Self::Reply, sails_rs::scale_codec::Error> {
                    Ok(())
                }
            }

            impl<E: GearEnv> $client_name<E> {
                pub fn constructor(
                    &self,
                    code_id: sails_rs::CodeId,
                    salt: impl AsRef<[u8]>,
                    encoded_args: impl AsRef<[u8]>,
                ) -> PendingCtor<(), Constructor, E> {
                    PendingCtor::new(
                        self.env.clone(),
                        code_id,
                        salt.as_ref().to_vec(),
                        encoded_args.as_ref().to_vec(),
                    )
                }
            }
        };
    }

    for (idx, func) in spec.functions.iter().enumerate() {
        let io_name = func.io_name.as_str();
        let method_name = func.method_name.as_str();
        let signature = &func.signature;
        let signature_const = func.signature_const.as_str();
        let entry_id = idx as u16;

        quote_in! { tokens =>
            pub struct $io_name;

            impl Identifiable for $io_name {
                const INTERFACE_ID: sails_rs::InterfaceId = sails_rs::InterfaceId::zero();
            }

            impl MethodMeta for $io_name {
                const ENTRY_ID: u16 = $entry_id;
            }

            impl ServiceCall for $io_name {
                type Params = sails_rs::prelude::Vec<u8>;
                type Reply = sails_rs::prelude::Vec<u8>;

                fn encode_params_with_header(
                    _route_idx: u8,
                    value: &Self::Params,
                ) -> sails_rs::prelude::Vec<u8> {
                    let selector = solidity::selector($(quoted(signature)));
                    [selector.as_slice(), value.as_slice()].concat()
                }

                fn decode_reply_with_header(
                    _route_idx: u8,
                    payload: impl AsRef<[u8]>,
                ) -> Result<Self::Reply, sails_rs::scale_codec::Error> {
                    Ok(payload.as_ref().to_vec())
                }
            }

            impl<E: GearEnv> $client_name<E> {
                pub const $signature_const: &'static str = $(quoted(signature));

                pub fn $method_name(
                    &self,
                    encoded_args: impl AsRef<[u8]>,
                ) -> PendingCall<$io_name, E> {
                    PendingCall::<$io_name, E>::new(
                        self.env.clone(),
                        self.program_id,
                        0,
                        encoded_args.as_ref().to_vec(),
                    )
                }
            }
        };
    }

    let mut output = tokens.to_file_string().expect("Failed to emit generated tokens");
    if with_no_std {
        output.insert_str(
            0,
            "// Code generated by sails-sol-client-gen. DO NOT EDIT.\n#![no_std]\n\n",
        );
    } else {
        output.insert_str(0, "// Code generated by sails-sol-client-gen. DO NOT EDIT.\n");
    }
    output
}

// not using prettyplease since it's bad at reporting syntax errors and also removes comments
fn pretty_with_rustfmt(code: &str) -> String {
    use std::process::Command;
    let mut child = Command::new("rustfmt")
        .arg("--edition")
        .arg("2024")
        .arg("--config")
        .arg("style_edition=2024")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn rustfmt");

    let child_stdin = child.stdin.as_mut().expect("Failed to open stdin");
    child_stdin
        .write_all(code.as_bytes())
        .expect("Failed to write to rustfmt");

    let output = child.wait_with_output().expect("Failed to wait for rustfmt");

    if !output.status.success() {
        panic!(
            "rustfmt failed with status: {}\n{}",
            output.status,
            String::from_utf8(output.stderr).expect("Failed to read rustfmt stderr")
        );
    }

    String::from_utf8(output.stdout).expect("Failed to read rustfmt output")
}
