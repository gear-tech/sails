use sails_idlparser::{types, types::visitor, types::visitor::Visitor, types::*};

pub fn process_file(content: &str) -> String {
    let program = types::parse_idl(content).unwrap();
    let mut trait_generator = RootGenerator::new("MyService");
    visitor::accept_program(&program, &mut trait_generator);
    let code = trait_generator.code;
    let code = syn::parse_str(&code).unwrap();
    prettyplease::unparse(&code)
}

struct RootGenerator<'a> {
    service_name: &'a str,
    code: String,
}

impl<'a> RootGenerator<'a> {
    fn new(service_name: &'a str) -> Self {
        Self {
            service_name,
            code: "use mockall::automock;".into(),
        }
    }
}

impl<'a, 'ast> Visitor<'ast> for RootGenerator<'a> {
    fn visit_service(&mut self, service: &'ast Service) {
        self.code
            .push_str(&format!("#[automock] trait {} {{", self.service_name));
        visitor::accept_service(service, self);
        self.code.push_str("}");
    }

    fn visit_func(&mut self, func: &'ast Func) {
        let mutability = if func.is_query() { "" } else { "mut" };
        self.code
            .push_str(&format!("async fn {}(&{} self,", func.name(), mutability));
        visitor::accept_func(func, self);
        self.code.push_str(";");
    }

    fn visit_func_param(&mut self, func_param: &'ast FuncParam) {
        let type_decl_code = generate_type_decl_code(func_param.type_decl());
        self.code
            .push_str(&format!("{}: {},", func_param.name(), type_decl_code));
    }

    fn visit_func_output(&mut self, func_output: &'ast TypeDecl) {
        let type_decl_code = generate_type_decl_code(func_output);
        self.code.push_str(&format!(") -> {}", type_decl_code));
    }

    fn visit_type(&mut self, r#type: &'ast Type) {
        let mut type_generator = TypeGenerator::new(r#type.name());
        type_generator.visit_type(r#type);
        self.code.push_str(&type_generator.code);
    }
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
        self.code.push_str(&format!(
            "struct {} {}",
            self.type_name, struct_def_generator.code
        ));
    }

    fn visit_enum_def(&mut self, enum_def: &'ast EnumDef) {
        let mut enum_def_generator = EnumDefGenerator::default();
        enum_def_generator.visit_enum_def(enum_def);
        self.code.push_str(&format!(
            "enum {} {}",
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
            self.code.push_str("{");
        } else {
            self.code.push_str("(");
        }
        visitor::accept_struct_def(struct_def, self);
        if is_regular_struct {
            self.code.push_str("}");
        } else {
            self.code.push_str(")");
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
        self.code.push_str("{");
        visitor::accept_enum_def(enum_def, self);
        self.code.push_str("}");
    }

    fn visit_enum_variant(&mut self, enum_variant: &'ast EnumVariant) {
        if let Some(type_decl) = enum_variant.type_decl().as_ref() {
            let type_decl_code = generate_type_decl_code(type_decl);
            if type_decl_code.starts_with("{") {
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
        self.code.push_str(&format!("Option<"));
        visitor::accept_type_decl(optional_type_decl, self);
        self.code.push_str(&format!(">"));
    }

    fn visit_result_type_decl(
        &mut self,
        ok_type_decl: &'ast TypeDecl,
        err_type_decl: &'ast TypeDecl,
    ) {
        self.code.push_str(&format!("Result<"));
        visitor::accept_type_decl(ok_type_decl, self);
        self.code.push_str(&format!(", "));
        visitor::accept_type_decl(err_type_decl, self);
        self.code.push_str(&format!(">"));
    }

    fn visit_vector_type_decl(&mut self, vector_type_decl: &'ast TypeDecl) {
        self.code.push_str(&format!("Vec<"));
        visitor::accept_type_decl(vector_type_decl, self);
        self.code.push_str(&format!(">"));
    }

    fn visit_struct_def(&mut self, struct_def: &'ast StructDef) {
        let mut struct_def_generator = StructDefGenerator::default();
        struct_def_generator.visit_struct_def(struct_def);
        self.code.push_str(&struct_def_generator.code);
    }

    fn visit_primitive_type_id(&mut self, primitive_type_id: &'ast PrimitiveType) {
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
        });
    }

    fn visit_user_defined_type_id(&mut self, user_defined_type_id: &'ast str) {
        self.code.push_str(&format!("{}", user_defined_type_id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_file_works() {
        let code = process_file(
            r"
            service {
                DoThis: (p1: u32, p2: MyParam) -> u16;
                DoThat: (p1: struct { u8, u32 }) -> u8;
            };

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
            ",
        );
        println!("{}", code);
    }
}
