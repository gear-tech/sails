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
    constructor: Option<IoSpec>,
    functions: Vec<FunctionSpec>,
}

#[derive(Debug)]
struct IoSpec {
    params: Vec<ParamSpec>,
    returns: Vec<syn_solidity::Type>,
}

#[derive(Debug)]
struct ParamSpec {
    name: String,
    ty: syn_solidity::Type,
}

#[derive(Debug)]
struct FunctionSpec {
    signature: String,
    method_name: String,
    io_name: String,
    signature_const: String,
    io: IoSpec,
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

    let mut constructor = None;
    let mut seen_names = HashSet::new();
    let mut functions = Vec::new();

    for item in &selected.body {
        let syn_solidity::Item::Function(function) = item else {
            continue;
        };

        match &function.kind {
            syn_solidity::FunctionKind::Constructor(_) => {
                if constructor.is_some() {
                    bail!(
                        "Multiple constructors found in `{}` declaration; only one constructor is supported",
                        selected.name
                    );
                }
                constructor = Some(IoSpec {
                    params: extract_params(&function.parameters, &["self", "code_id", "salt"])?,
                    returns: Vec::new(),
                });
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
                    io: IoSpec {
                        params: extract_params(&function.parameters, &["self"])?,
                        returns: function
                            .returns
                            .as_ref()
                            .map(|returns| returns.returns.types().cloned().collect())
                            .unwrap_or_default(),
                    },
                });
            }
            _ => {}
        }
    }

    if functions.is_empty() && constructor.is_none() {
        bail!(
            "No callable `function` or `constructor` items found in `{}` declaration",
            selected.name
        );
    }

    Ok(ContractSpec {
        rust_name: selected.name.to_string().to_case(Case::Pascal),
        constructor,
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
            client::{GearEnv, PendingCall, PendingCtor},
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

    if let Some(constructor) = &spec.constructor {
        let constructor_param_types = param_types(&constructor.params);
        let params_type =
            rust_tuple_type(&constructor_param_types).expect("supported constructor params");
        let abi_params_type = rust_abi_tuple_type(&constructor_param_types)
            .expect("supported constructor abi params");
        let abi_params_type_for_into = abi_params_type.clone();
        let encode_params_expr = if needs_abi_conversion_for_tuple(&constructor_param_types) {
            borrowed_tuple_to_abi_expr(&constructor_param_types, "value")
                .expect("supported constructor param conversion")
        } else {
            "value.clone()".to_string()
        };
        let constructor_args_signature = method_args_signature(&constructor.params)
            .expect("supported constructor argument signature");
        let constructor_args_tuple =
            args_tuple_expr(&constructor.params).expect("supported constructor args tuple");

        quote_in! { tokens =>
            sails_rs::io_struct_sol_impl!(Constructor, u16::MAX, {
                type Params = $params_type;
                type AbiParams = $abi_params_type;
                into_abi_params = |value| {
                    let abi_value: $abi_params_type_for_into = $encode_params_expr;
                    abi_value
                };
                type Reply = ();
                type AbiReply = ();
                from_abi_reply = |value| { value };
                call = constructor;
                reply = unit;
            });

            impl<E: GearEnv> $client_name<E> {
                pub fn constructor(
                    &self,
                    code_id: sails_rs::CodeId,
                    salt: impl AsRef<[u8]>,
                    $constructor_args_signature
                ) -> PendingCtor<(), Constructor, E> {
                    PendingCtor::new(
                        self.env.clone(),
                        code_id,
                        salt.as_ref().to_vec(),
                        $constructor_args_tuple,
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
        let function_param_types = param_types(&func.io.params);
        let params_type =
            rust_tuple_type(&function_param_types).expect("supported function params");
        let abi_params_type = rust_abi_tuple_type(&function_param_types)
            .expect("supported function abi params");
        let abi_params_type_for_into = abi_params_type.clone();
        let reply_type = rust_reply_type(&func.io.returns).expect("supported function reply");
        let abi_reply_type =
            rust_abi_reply_type(&func.io.returns).expect("supported function abi reply");
        let encode_params_expr = if needs_abi_conversion_for_tuple(&function_param_types) {
            borrowed_tuple_to_abi_expr(&function_param_types, "value")
                .expect("supported function param conversion")
        } else {
            "value.clone()".to_string()
        };
        let decode_reply_kind = reply_decode_kind(&func.io.returns);
        let from_abi_reply_expr = from_reply_expr(&func.io.returns).expect("supported function reply");
        let method_args_signature =
            method_args_signature(&func.io.params).expect("supported function arg signature");
        let method_args_tuple = args_tuple_expr(&func.io.params).expect("supported function arg tuple");

        quote_in! { tokens =>
            sails_rs::io_struct_sol_impl!($io_name, $entry_id, {
                type Params = $params_type;
                type AbiParams = $abi_params_type;
                into_abi_params = |value| {
                    let abi_value: $abi_params_type_for_into = $encode_params_expr;
                    abi_value
                };
                type Reply = $reply_type;
                type AbiReply = $abi_reply_type;
                from_abi_reply = |value| {
                    $from_abi_reply_expr
                };
                call = selector($(quoted(signature)));
                reply = $decode_reply_kind;
            });

            impl<E: GearEnv> $client_name<E> {
                pub const $signature_const: &'static str = $(quoted(signature));

                pub fn $method_name(&self, $method_args_signature) -> PendingCall<$io_name, E> {
                    PendingCall::<$io_name, E>::new(
                        self.env.clone(),
                        self.program_id,
                        0,
                        $method_args_tuple,
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

fn extract_params(
    parameters: &syn_solidity::ParameterList,
    reserved_names: &[&str],
) -> Result<Vec<ParamSpec>> {
    let mut used_names: HashSet<String> = reserved_names.iter().map(|name| (*name).to_string()).collect();

    parameters
        .types_and_names()
        .enumerate()
        .map(|(index, (ty, name))| {
            Ok(ParamSpec {
                name: unique_param_name(normalize_param_name(name, index), &mut used_names),
                ty: ty.clone(),
            })
        })
        .collect()
}

fn normalize_param_name(name: Option<&syn_solidity::SolIdent>, index: usize) -> String {
    let normalized = name
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("param_{}", index + 1))
        .trim_start_matches('_')
        .to_case(Case::Snake);
    let normalized = collapse_digit_boundaries(&normalized);

    let normalized = match normalized.as_str() {
        "" => format!("param_{}", index + 1),
        "call_reply" => "encode_reply".to_string(),
        other => other.to_string(),
    };

    let normalized = if normalized
        .chars()
        .next()
        .map(|c| c.is_ascii_digit())
        .unwrap_or(false)
    {
        format!("param_{normalized}")
    } else {
        normalized
    };

    match normalized.as_str() {
        "self" | "crate" | "super" | "Self" | "fn" | "move" | "type" | "where" | "use"
        | "pub" | "mod" | "enum" | "struct" | "trait" | "impl" | "let" | "match" | "loop"
        | "for" | "while" | "async" | "await" | "dyn" => format!("r#{normalized}"),
        _ => normalized,
    }
}

fn collapse_digit_boundaries(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '_'
            && result.chars().last().map(|c| c.is_ascii_alphanumeric()).unwrap_or(false)
            && chars.peek().copied().map(|c| c.is_ascii_digit()).unwrap_or(false)
        {
            continue;
        }
        result.push(ch);
    }
    result
}

fn unique_param_name(base: String, used_names: &mut HashSet<String>) -> String {
    if used_names.insert(base.clone()) {
        return base;
    }

    for index in 2.. {
        let candidate = format!("{base}_{index}");
        if used_names.insert(candidate.clone()) {
            return candidate;
        }
    }

    unreachable!("infinite suffix space exhausted")
}

fn param_types(params: &[ParamSpec]) -> Vec<syn_solidity::Type> {
    params.iter().map(|param| param.ty.clone()).collect()
}

fn method_args_signature(params: &[ParamSpec]) -> Result<String> {
    Ok(params
        .iter()
        .map(|param| Ok(format!("{}: {}", param.name, rust_public_type(&param.ty)?)))
        .collect::<Result<Vec<_>>>()?
        .join(", "))
}

fn args_tuple_expr(params: &[ParamSpec]) -> Result<String> {
    Ok(match params {
        [] => "()".to_string(),
        [param] => format!("({},)", param.name),
        _ => format!(
            "({})",
            params
                .iter()
                .map(|param| param.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    })
}

fn rust_tuple_type(types: &[syn_solidity::Type]) -> Result<String> {
    Ok(tuple_type(types, rust_public_type)?)
}

fn rust_abi_tuple_type(types: &[syn_solidity::Type]) -> Result<String> {
    Ok(tuple_type(types, rust_abi_type)?)
}

fn rust_reply_type(types: &[syn_solidity::Type]) -> Result<String> {
    match types {
        [] => Ok("()".to_string()),
        [ty] => rust_public_type(ty),
        _ => tuple_type(types, rust_public_type),
    }
}

fn rust_abi_reply_type(types: &[syn_solidity::Type]) -> Result<String> {
    match types {
        [] => Ok("()".to_string()),
        [ty] => rust_abi_type(ty),
        _ => tuple_type(types, rust_abi_type),
    }
}

fn tuple_type(
    types: &[syn_solidity::Type],
    mapper: impl Fn(&syn_solidity::Type) -> Result<String>,
) -> Result<String> {
    let mapped = types.iter().map(mapper).collect::<Result<Vec<_>>>()?;
    Ok(match mapped.as_slice() {
        [] => "()".to_string(),
        [single] => format!("({single},)"),
        _ => format!("({})", mapped.join(", ")),
    })
}

fn rust_public_type(ty: &syn_solidity::Type) -> Result<String> {
    match ty {
        syn_solidity::Type::Address(_, _) => Ok("sails_rs::ActorId".to_string()),
        syn_solidity::Type::Bool(_) => Ok("bool".to_string()),
        syn_solidity::Type::String(_) => Ok("sails_rs::prelude::String".to_string()),
        syn_solidity::Type::Bytes(_) => Ok("sails_rs::prelude::Vec<u8>".to_string()),
        syn_solidity::Type::FixedBytes(_, size) => Ok(format!("[u8; {}]", size.get())),
        syn_solidity::Type::Uint(_, size) => Ok(match size.map(|s| s.get()).unwrap_or(256) {
            8 => "u8".to_string(),
            16 => "u16".to_string(),
            32 => "u32".to_string(),
            64 => "u64".to_string(),
            128 => "u128".to_string(),
            _ => "sails_rs::U256".to_string(),
        }),
        syn_solidity::Type::Int(_, size) => Ok(match size.map(|s| s.get()).unwrap_or(256) {
            8 => "i8".to_string(),
            16 => "i16".to_string(),
            32 => "i32".to_string(),
            64 => "i64".to_string(),
            128 => "i128".to_string(),
            bits => bail!("Unsupported signed Solidity integer type `int{bits}` for generated client"),
        }),
        syn_solidity::Type::Array(array) => {
            let inner = rust_public_type(&array.ty)?;
            Ok(match array.size() {
                Some(size) => format!("[{inner}; {size}]"),
                None => format!("sails_rs::prelude::Vec<{inner}>"),
            })
        }
        syn_solidity::Type::Tuple(tuple) => tuple_type(&tuple.types.iter().cloned().collect::<Vec<_>>(), rust_public_type),
        syn_solidity::Type::Custom(_) => Ok("sails_rs::ActorId".to_string()),
        syn_solidity::Type::Function(_) | syn_solidity::Type::Mapping(_) => bail!(
            "Unsupported Solidity type `{ty}` in generated client"
        ),
    }
}

fn rust_abi_type(ty: &syn_solidity::Type) -> Result<String> {
    match ty {
        syn_solidity::Type::Address(_, _) => Ok("sails_rs::ActorId".to_string()),
        syn_solidity::Type::Bool(_) => Ok("bool".to_string()),
        syn_solidity::Type::String(_) => Ok("sails_rs::prelude::String".to_string()),
        syn_solidity::Type::Bytes(_) => Ok("sails_rs::prelude::Vec<u8>".to_string()),
        syn_solidity::Type::FixedBytes(_, size) => Ok(format!("[u8; {}]", size.get())),
        syn_solidity::Type::Uint(_, size) => Ok(match size.map(|s| s.get()).unwrap_or(256) {
            8 => "u8".to_string(),
            16 => "u16".to_string(),
            32 => "u32".to_string(),
            64 => "u64".to_string(),
            128 => "u128".to_string(),
            _ => "sails_rs::alloy_primitives::U256".to_string(),
        }),
        syn_solidity::Type::Int(_, size) => Ok(match size.map(|s| s.get()).unwrap_or(256) {
            8 => "i8".to_string(),
            16 => "i16".to_string(),
            32 => "i32".to_string(),
            64 => "i64".to_string(),
            128 => "i128".to_string(),
            bits => bail!("Unsupported signed Solidity integer type `int{bits}` for generated client"),
        }),
        syn_solidity::Type::Array(array) => {
            let inner = rust_abi_type(&array.ty)?;
            Ok(match array.size() {
                Some(size) => format!("[{inner}; {size}]"),
                None => format!("sails_rs::prelude::Vec<{inner}>"),
            })
        }
        syn_solidity::Type::Tuple(tuple) => tuple_type(&tuple.types.iter().cloned().collect::<Vec<_>>(), rust_abi_type),
        syn_solidity::Type::Custom(_) => Ok("sails_rs::ActorId".to_string()),
        syn_solidity::Type::Function(_) | syn_solidity::Type::Mapping(_) => bail!(
            "Unsupported Solidity type `{ty}` in generated client"
        ),
    }
}

fn needs_abi_conversion_for_tuple(types: &[syn_solidity::Type]) -> bool {
    types.iter().any(needs_abi_conversion)
}

fn needs_abi_conversion(ty: &syn_solidity::Type) -> bool {
    match ty {
        syn_solidity::Type::Uint(_, size) => !matches!(size.map(|s| s.get()).unwrap_or(256), 8 | 16 | 32 | 64 | 128),
        syn_solidity::Type::Array(array) => needs_abi_conversion(&array.ty),
        syn_solidity::Type::Tuple(tuple) => tuple.types.iter().any(needs_abi_conversion),
        _ => false,
    }
}

fn borrowed_tuple_to_abi_expr(types: &[syn_solidity::Type], value_expr: &str) -> Result<String> {
    match types {
        [] => Ok("()".to_string()),
        [single] => Ok(format!(
            "({},)",
            borrowed_to_abi_expr(single, &format!("&{value_expr}.0"))?
        )),
        _ => Ok(format!(
            "({})",
            types
                .iter()
                .enumerate()
                .map(|(index, ty)| borrowed_to_abi_expr(ty, &format!("&{value_expr}.{index}")))
                .collect::<Result<Vec<_>>>()?
                .join(", ")
        )),
    }
}

fn borrowed_to_abi_expr(ty: &syn_solidity::Type, value_expr: &str) -> Result<String> {
    if !needs_abi_conversion(ty) {
        return Ok(format!("({value_expr}).clone()"));
    }

    match ty {
        syn_solidity::Type::Uint(_, _) => Ok(format!(
            "sails_rs::alloy_primitives::U256::from_be_bytes::<32>({{ let mut bytes = [0u8; 32]; ({value_expr}).to_big_endian(&mut bytes); bytes }})"
        )),
        syn_solidity::Type::Array(array) => {
            let inner = &array.ty;
            Ok(match array.size() {
                Some(_) => format!(
                    "core::array::from_fn(|i| {{ let item = &{value_expr}[i]; {} }})",
                    borrowed_to_abi_expr(inner, "item")?
                ),
                None => format!(
                    "{value_expr}.iter().map(|item| {}).collect::<sails_rs::prelude::Vec<_>>()",
                    borrowed_to_abi_expr(inner, "item")?
                ),
            })
        }
        syn_solidity::Type::Tuple(tuple) => Ok(format!(
            "({})",
            tuple.types
                .iter()
                .enumerate()
                .map(|(index, inner)| borrowed_to_abi_expr(inner, &format!("&{value_expr}.{index}")))
                .collect::<Result<Vec<_>>>()?
                .join(", ")
        )),
        _ => Ok(format!("{value_expr}.clone()")),
    }
}

fn from_abi_expr(ty: &syn_solidity::Type, value_expr: &str) -> Result<String> {
    if !needs_abi_conversion(ty) {
        return Ok(value_expr.to_string());
    }

    match ty {
        syn_solidity::Type::Uint(_, _) => Ok(format!(
            "sails_rs::U256::from_big_endian(&{value_expr}.to_be_bytes::<32>())"
        )),
        syn_solidity::Type::Array(array) => {
            let inner = &array.ty;
            Ok(match array.size() {
                Some(_) => format!("{value_expr}.map(|item| {})", from_abi_expr(inner, "item")?),
                None => format!(
                    "{value_expr}.into_iter().map(|item| {}).collect::<sails_rs::prelude::Vec<_>>()",
                    from_abi_expr(inner, "item")?
                ),
            })
        }
        syn_solidity::Type::Tuple(tuple) => Ok(match tuple.types.len() {
            1 => format!("({},)", from_abi_expr(&tuple.types[0], &format!("{value_expr}.0"))?),
            _ => format!(
                "({})",
                tuple
                    .types
                    .iter()
                    .enumerate()
                    .map(|(index, inner)| from_abi_expr(inner, &format!("{value_expr}.{index}")))
                    .collect::<Result<Vec<_>>>()?
                    .join(", ")
            ),
        }),
        _ => Ok(value_expr.to_string()),
    }
}

fn reply_decode_kind(types: &[syn_solidity::Type]) -> &'static str {
    match types {
        [] => "unit",
        [_] => "value",
        _ => "sequence",
    }
}

fn from_reply_expr(types: &[syn_solidity::Type]) -> Result<String> {
    match types {
        [] => Ok("value".to_string()),
        [single] => {
            if needs_abi_conversion(single) {
                from_abi_expr(single, "value")
            } else {
                Ok("value".to_string())
            }
        }
        _ => {
            let tuple_ty = syn_solidity::Type::Tuple(syn_solidity::TypeTuple::from_iter(types.iter().cloned()));
            if needs_abi_conversion(&tuple_ty) {
                from_abi_expr(&tuple_ty, "value")
            } else {
                Ok("value".to_string())
            }
        }
    }
}
