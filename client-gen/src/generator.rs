use anyhow::Result;
use convert_case::{Case, Casing};
use sails_idlparser::ast::*;
use sails_idlparser::{ast::visitor, ast::visitor::Visitor};
use std::io::Write;

pub fn generate(program: Program) -> Result<String> {
    let mut trait_generator = RootGenerator::new("Service");
    visitor::accept_program(&program, &mut trait_generator);

    let code = trait_generator.code;

    // Check for parsing errors
    let code = pretty_with_rustfmt(&code);

    Ok(code)
}

// not using prettyplease since it's bad at reporting syntax errors and also removes comments
// TODO(holykol): Fallback if rustfmt is not in PATH would be nice
fn pretty_with_rustfmt(code: &str) -> String {
    use std::process::Command;
    let mut child = Command::new("rustfmt")
        .arg("--config")
        .arg("format_strings=false")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn rustfmt");

    let child_stdin = child.stdin.as_mut().expect("Failed to open stdin");
    child_stdin
        .write_all(code.as_bytes())
        .expect("Failed to write to rustfmt");

    let output = child
        .wait_with_output()
        .expect("Failed to wait for rustfmt");

    if !output.status.success() {
        panic!(
            "rustfmt failed with status: {}\n{}",
            output.status,
            String::from_utf8(output.stderr).expect("Failed to read rustfmt stderr")
        );
    }

    String::from_utf8(output.stdout).expect("Failed to read rustfmt output")
}

struct RootGenerator<'a> {
    code: String,
    service_name: &'a str,
}

impl<'a> RootGenerator<'a> {
    fn new(service_name: &'a str) -> Self {
        let mut code = String::new();

        code.push_str("// Code generated by sails-client-gen. DO NOT EDIT.\n");

        code.push_str("use parity_scale_codec::{Encode, Decode};\n");
        code.push_str("use sails_rtl::{*, String};\n");
        code.push_str("use sails_sender::{Call, GStdSender};\n");
        code.push_str("#[allow(unused_imports)]\n");
        code.push_str("use sails_rtl::collections::BTreeMap;\n");

        Self { service_name, code }
    }
}

impl<'a, 'ast> Visitor<'ast> for RootGenerator<'a> {
    fn visit_service(&mut self, service: &'ast Service) {
        let mut service_gen = ServiceGenerator::new(self.service_name.to_owned());
        service_gen.visit_service(service);
        self.code.push_str(&service_gen.code);

        let mut client_gen = ClientGenerator::new(self.service_name.to_owned());
        client_gen.visit_service(service);
        self.code.push_str(&client_gen.code);
    }

    fn visit_type(&mut self, t: &'ast Type) {
        let mut type_gen = TypeGenerator::new(t.name());
        type_gen.visit_type(t);
        self.code.push_str(&type_gen.code);
    }
}

struct ServiceGenerator {
    service_name: String,
    code: String,
}

impl ServiceGenerator {
    fn new(service_name: String) -> Self {
        Self {
            service_name,
            code: String::new(),
        }
    }
}

impl<'ast> Visitor<'ast> for ServiceGenerator {
    fn visit_service(&mut self, service: &'ast Service) {
        self.code
            .push_str(&format!("pub trait {} {{\n", self.service_name));
        visitor::accept_service(service, self);
        self.code.push_str("}\n");
    }

    fn visit_service_func(&mut self, func: &'ast ServiceFunc) {
        let mutability = if func.is_query() { "" } else { "mut" };
        let name = func.name().to_case(Case::Snake);

        self.code
            .push_str(&format!("fn {}(&{} self,", name, mutability));
        visitor::accept_service_func(func, self);
        self.code.push_str(";\n");
    }

    fn visit_func_param(&mut self, func_param: &'ast FuncParam) {
        let type_decl_code = generate_type_decl_code(func_param.type_decl());
        self.code
            .push_str(&format!("{}: {},", func_param.name(), type_decl_code));
    }

    fn visit_func_output(&mut self, func_output: &'ast TypeDecl) {
        let type_decl_code = generate_type_decl_code(func_output);
        self.code
            .push_str(&format!(") -> Call<{}>", type_decl_code));
    }
}

struct ClientGenerator {
    service_name: String,
    code: String,
}

impl ClientGenerator {
    fn new(service_name: String) -> Self {
        Self {
            service_name,
            code: String::new(),
        }
    }
}

impl<'ast> Visitor<'ast> for ClientGenerator {
    fn visit_service(&mut self, service: &'ast Service) {
        let name = &self.service_name;

        self.code.push_str(&format!(
            r#"
            #[derive(Clone)]
            pub struct Client {{
                sender: GStdSender
            }}

            impl Client {{
                pub fn new(sender: GStdSender) -> Self {{
                    Self {{ sender }}
                }}
            }}

            impl {name} for Client {{
        "#
        ));

        visitor::accept_service(service, self);
        self.code.push_str("}\n");
    }

    fn visit_service_func(&mut self, func: &'ast ServiceFunc) {
        let mutability = if func.is_query() { "" } else { "mut" };
        let fn_name = func.name();

        self.code.push_str(&format!(
            "fn {}(&{} self,",
            fn_name.to_case(Case::Snake),
            mutability
        ));

        visitor::accept_service_func(func, self);

        self.code.push_str("{\n");

        let args = encoded_args(func.params());

        self.code
            .push_str(&format!("Call::new(&self.sender, \"{fn_name}\", {args})"));

        self.code.push_str("}\n");
    }

    fn visit_func_param(&mut self, func_param: &'ast FuncParam) {
        let type_decl_code = generate_type_decl_code(func_param.type_decl());
        self.code
            .push_str(&format!("{}: {},", func_param.name(), type_decl_code));
    }

    fn visit_func_output(&mut self, func_output: &'ast TypeDecl) {
        let type_decl_code = generate_type_decl_code(func_output);
        self.code
            .push_str(&format!(") -> Call<{}>", type_decl_code));
    }
}

fn encoded_args(params: &[FuncParam]) -> String {
    if params.len() == 1 {
        return params[0].name().to_owned();
    }

    let arg_names = params
        .iter()
        .map(|a| a.name())
        .collect::<Vec<_>>()
        .join(", ");

    format!("({arg_names})")
}

struct TypeGenerator<'a> {
    type_name: &'a str,
    code: String,
}

impl<'a> TypeGenerator<'a> {
    fn new(type_name: &'a str) -> Self {
        Self {
            type_name,
            code: String::new(),
        }
    }
}

impl<'a, 'ast> Visitor<'ast> for TypeGenerator<'a> {
    fn visit_struct_def(&mut self, struct_def: &'ast StructDef) {
        let mut struct_def_generator = StructDefGenerator::default();
        struct_def_generator.visit_struct_def(struct_def);

        let semi = if struct_def.fields().iter().all(|f| f.name().is_none()) {
            ";"
        } else {
            ""
        };

        self.code
            .push_str("#[derive(PartialEq, Debug, Encode, Decode)]");
        self.code.push_str(&format!(
            "pub struct {} {} {}",
            self.type_name, struct_def_generator.code, semi
        ));
    }

    fn visit_enum_def(&mut self, enum_def: &'ast EnumDef) {
        let mut enum_def_generator = EnumDefGenerator::default();
        enum_def_generator.visit_enum_def(enum_def);

        self.code
            .push_str("#[derive(PartialEq, Debug, Encode, Decode)]");
        self.code.push_str(&format!(
            "pub enum {} {}",
            self.type_name, &enum_def_generator.code
        ));
    }
}

#[derive(Default)]
struct StructDefGenerator {
    code: String,
}

impl<'ast> Visitor<'ast> for StructDefGenerator {
    fn visit_struct_def(&mut self, struct_def: &'ast StructDef) {
        let is_regular_struct = struct_def.fields().iter().all(|f| f.name().is_some());
        let is_tuple_struct = struct_def.fields().iter().all(|f| f.name().is_none());
        if !is_regular_struct && !is_tuple_struct {
            panic!("Struct must be either regular or tuple");
        }
        if is_regular_struct {
            self.code.push('{');
        } else {
            self.code.push('(');
        }
        visitor::accept_struct_def(struct_def, self);
        if is_regular_struct {
            self.code.push('}');
        } else {
            self.code.push(')');
        }
    }

    fn visit_struct_field(&mut self, struct_field: &'ast StructField) {
        let type_decl_code = generate_type_decl_code(struct_field.type_decl());

        if let Some(field_name) = struct_field.name() {
            self.code
                .push_str(&format!("{}: {},", field_name, type_decl_code));
        } else {
            self.code.push_str(&format!("{},", type_decl_code));
        }
    }
}

#[derive(Default)]
struct EnumDefGenerator {
    code: String,
}

impl<'ast> Visitor<'ast> for EnumDefGenerator {
    fn visit_enum_def(&mut self, enum_def: &'ast EnumDef) {
        self.code.push('{');
        visitor::accept_enum_def(enum_def, self);
        self.code.push('}');
    }

    fn visit_enum_variant(&mut self, enum_variant: &'ast EnumVariant) {
        if let Some(type_decl) = enum_variant.type_decl().as_ref() {
            let type_decl_code = generate_type_decl_code(type_decl);
            if type_decl_code.starts_with('{') {
                self.code
                    .push_str(&format!("{} {},", enum_variant.name(), type_decl_code));
            } else {
                self.code
                    .push_str(&format!("{}({}),", enum_variant.name(), type_decl_code));
            }
        } else {
            self.code.push_str(&format!("{},", enum_variant.name()));
        }
    }
}

fn generate_type_decl_code(type_decl: &TypeDecl) -> String {
    let mut type_decl_generator = TypeDeclGenerator::default();
    visitor::accept_type_decl(type_decl, &mut type_decl_generator);
    type_decl_generator.code
}

#[derive(Default)]
struct TypeDeclGenerator {
    code: String,
}

impl<'ast> Visitor<'ast> for TypeDeclGenerator {
    fn visit_optional_type_decl(&mut self, optional_type_decl: &'ast TypeDecl) {
        self.code.push_str("Option<");
        visitor::accept_type_decl(optional_type_decl, self);
        self.code.push('>');
    }

    fn visit_result_type_decl(
        &mut self,
        ok_type_decl: &'ast TypeDecl,
        err_type_decl: &'ast TypeDecl,
    ) {
        self.code.push_str("Result<");
        visitor::accept_type_decl(ok_type_decl, self);
        self.code.push_str(", ");
        visitor::accept_type_decl(err_type_decl, self);
        self.code.push('>');
    }

    fn visit_vector_type_decl(&mut self, vector_type_decl: &'ast TypeDecl) {
        self.code.push_str("Vec<");
        visitor::accept_type_decl(vector_type_decl, self);
        self.code.push('>');
    }

    fn visit_struct_def(&mut self, struct_def: &'ast StructDef) {
        let mut struct_def_generator = StructDefGenerator::default();
        struct_def_generator.visit_struct_def(struct_def);
        self.code.push_str(&struct_def_generator.code);
    }

    fn visit_primitive_type_id(&mut self, primitive_type_id: PrimitiveType) {
        self.code.push_str(match primitive_type_id {
            PrimitiveType::U8 => "u8",
            PrimitiveType::U16 => "u16",
            PrimitiveType::U32 => "u32",
            PrimitiveType::U64 => "u64",
            PrimitiveType::U128 => "u128",
            PrimitiveType::I8 => "i8",
            PrimitiveType::I16 => "i16",
            PrimitiveType::I32 => "i32",
            PrimitiveType::I64 => "i64",
            PrimitiveType::I128 => "i128",
            PrimitiveType::Bool => "bool",
            PrimitiveType::Str => "String",
            PrimitiveType::Char => "char",
            PrimitiveType::Null => "()",
            PrimitiveType::ActorId => "ActorId",
            PrimitiveType::CodeId => "CodeId",
            PrimitiveType::MessageId => "MessageId",
        });
    }

    fn visit_user_defined_type_id(&mut self, user_defined_type_id: &'ast str) {
        self.code.push_str(user_defined_type_id);
    }

    fn visit_map_type_decl(
        &mut self,
        key_type_decl: &'ast TypeDecl,
        value_type_decl: &'ast TypeDecl,
    ) {
        self.code.push_str("BTreeMap<");
        visitor::accept_type_decl(key_type_decl, self);
        self.code.push_str(", ");
        visitor::accept_type_decl(value_type_decl, self);
        self.code.push('>');
    }

    fn visit_array_type_decl(&mut self, item_type_decl: &'ast TypeDecl, len: u32) {
        self.code.push('[');
        visitor::accept_type_decl(item_type_decl, self);
        self.code.push_str(&format!("; {len}]"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_works() {
        let idl = r"
            type MyParam = struct {
                f1: u32,
                f2: vec str,
                f3: opt struct { u8, u32 },
            };

            type MyParam2 = enum {
                Variant1,
                Variant2: u32,
                Variant3: struct { u32 },
                Variant4: struct { u8, u32 },
                Variant5: struct { f1: str, f2: vec u8 },
            };

            service {
                DoThis: (p1: u32, p2: MyParam) -> u16;
                DoThat: (p1: struct { u8, u32 }) -> u8;
            };
        ";

        let program = sails_idlparser::ast::parse_idl(idl).expect("parse IDL");

        insta::assert_snapshot!(generate(program).unwrap());
    }

    #[test]
    fn test_rmrk_works() {
        let idl = include_str!("../../examples/rmrk/catalog/wasm/rmrk-catalog.idl");

        let program = sails_idlparser::ast::parse_idl(idl).expect("parse IDL");

        insta::assert_snapshot!(generate(program).unwrap());
    }
}
