use super::*;

pub trait Visitor<'ast> {
    fn visit_service(&mut self, service: &'ast Service) {
        accept_service(service, self);
    }

    fn visit_type(&mut self, r#type: &'ast Type) {
        accept_type(r#type, self);
    }

    fn visit_vector_type_decl(&mut self, item_type_decl: &'ast TypeDecl) {
        accept_type_decl(item_type_decl, self);
    }

    fn visit_array_type_decl(&mut self, item_type_decl: &'ast TypeDecl, _len: u32) {
        accept_type_decl(item_type_decl, self);
    }

    fn visit_map_type_decl(
        &mut self,
        key_type_decl: &'ast TypeDecl,
        value_type_decl: &'ast TypeDecl,
    ) {
        accept_type_decl(key_type_decl, self);
        accept_type_decl(value_type_decl, self);
    }

    fn visit_optional_type_decl(&mut self, optional_type_decl: &'ast TypeDecl) {
        accept_type_decl(optional_type_decl, self);
    }

    fn visit_result_type_decl(
        &mut self,
        ok_type_decl: &'ast TypeDecl,
        err_type_decl: &'ast TypeDecl,
    ) {
        accept_type_decl(ok_type_decl, self);
        accept_type_decl(err_type_decl, self);
    }

    fn visit_primitive_type_id(&mut self, _primitive_type_id: PrimitiveType) {}

    fn visit_user_defined_type_id(&mut self, _user_defined_type_id: &'ast str) {}

    fn visit_func(&mut self, func: &'ast Func) {
        accept_func(func, self);
    }

    fn visit_func_param(&mut self, func_param: &'ast FuncParam) {
        accept_func_param(func_param, self);
    }

    fn visit_func_output(&mut self, func_output: &'ast TypeDecl) {
        accept_type_decl(func_output, self);
    }

    fn visit_struct_def(&mut self, struct_def: &'ast StructDef) {
        accept_struct_def(struct_def, self);
    }

    fn visit_struct_field(&mut self, struct_field: &'ast StructField) {
        accept_struct_field(struct_field, self);
    }

    fn visit_enum_def(&mut self, enum_def: &'ast EnumDef) {
        accept_enum_def(enum_def, self);
    }

    fn visit_enum_variant(&mut self, enum_variant: &'ast EnumVariant) {
        accept_enum_variant(enum_variant, self);
    }
}

pub fn accept_program<'ast>(program: &'ast Program, visitor: &mut (impl Visitor<'ast> + ?Sized)) {
    visitor.visit_service(program.service());
    for r#type in program.types() {
        visitor.visit_type(r#type);
    }
}

pub fn accept_service<'ast>(service: &'ast Service, visitor: &mut (impl Visitor<'ast> + ?Sized)) {
    for func in service.funcs() {
        visitor.visit_func(func);
    }
}

pub fn accept_func<'ast>(func: &'ast Func, visitor: &mut (impl Visitor<'ast> + ?Sized)) {
    for param in func.params() {
        visitor.visit_func_param(param);
    }
    visitor.visit_func_output(func.output());
}

pub fn accept_func_param<'ast>(
    func_param: &'ast FuncParam,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    accept_type_decl(func_param.type_decl(), visitor);
}

pub fn accept_type<'ast>(r#type: &'ast Type, visitor: &mut (impl Visitor<'ast> + ?Sized)) {
    accept_type_def(r#type.def(), visitor);
}

pub fn accept_type_decl<'ast>(
    type_decl: &'ast TypeDecl,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    match type_decl {
        TypeDecl::Vector(item_type_decl) => {
            visitor.visit_vector_type_decl(item_type_decl);
        }
        TypeDecl::Array { item, len } => {
            visitor.visit_array_type_decl(item, *len);
        }
        TypeDecl::Map { key, value } => {
            visitor.visit_map_type_decl(key, value);
        }
        TypeDecl::Optional(optional_type_decl) => {
            visitor.visit_optional_type_decl(optional_type_decl);
        }
        TypeDecl::Result { ok, err } => {
            visitor.visit_result_type_decl(ok, err);
        }
        TypeDecl::Id(TypeId::Primitive(primitive_type_id)) => {
            visitor.visit_primitive_type_id(*primitive_type_id);
        }
        TypeDecl::Id(TypeId::UserDefined(user_defined_type_id)) => {
            visitor.visit_user_defined_type_id(user_defined_type_id);
        }
        TypeDecl::Def(type_def) => {
            accept_type_def(type_def, visitor);
        }
    }
}

pub fn accept_type_def<'ast>(type_def: &'ast TypeDef, visitor: &mut (impl Visitor<'ast> + ?Sized)) {
    match type_def {
        TypeDef::Struct(struct_def) => {
            visitor.visit_struct_def(struct_def);
        }
        TypeDef::Enum(enum_def) => {
            visitor.visit_enum_def(enum_def);
        }
    }
}

pub fn accept_struct_def<'ast>(
    struct_def: &'ast StructDef,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    for field in struct_def.fields() {
        visitor.visit_struct_field(field);
    }
}

pub fn accept_struct_field<'ast>(
    struct_field: &'ast StructField,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    accept_type_decl(struct_field.type_decl(), visitor);
}

pub fn accept_enum_def<'ast>(enum_def: &'ast EnumDef, visitor: &mut (impl Visitor<'ast> + ?Sized)) {
    for variant in enum_def.variants() {
        visitor.visit_enum_variant(variant);
    }
}

pub fn accept_enum_variant<'ast>(
    enum_variant: &'ast EnumVariant,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    if let Some(type_decl) = enum_variant.type_decl().as_ref() {
        accept_type_decl(type_decl, visitor);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ProgramVisitor<'ast> {
        service: Option<&'ast Service>,
        types: Vec<&'ast Type>,
    }

    impl<'ast> Visitor<'ast> for ProgramVisitor<'ast> {
        fn visit_service(&mut self, service: &'ast Service) {
            self.service = Some(service);
        }

        fn visit_type(&mut self, r#type: &'ast Type) {
            self.types.push(r#type);
        }
    }

    #[test]
    fn accept_program_works() {
        let program = Program::new(
            Service::new(vec![]),
            vec![
                Type::new("Type1".into(), TypeDef::Struct(StructDef::new(vec![]))),
                Type::new("Type2".into(), TypeDef::Enum(EnumDef::new(vec![]))),
            ],
        );
        let mut program_visitor = ProgramVisitor {
            service: None,
            types: vec![],
        };

        accept_program(&program, &mut program_visitor);

        assert_eq!(program_visitor.service, Some(program.service()));
        assert!(itertools::equal(program_visitor.types, program.types()));
    }

    struct EnumDefVisitor<'ast> {
        variants: Vec<&'ast EnumVariant>,
    }

    impl<'ast> Visitor<'ast> for EnumDefVisitor<'ast> {
        fn visit_enum_variant(&mut self, enum_variant: &'ast EnumVariant) {
            self.variants.push(enum_variant);
        }
    }

    #[test]
    fn accept_enum_def_works() {
        let enum_def = EnumDef::new(vec![
            EnumVariant::new("Variant1".into(), None),
            EnumVariant::new(
                "Variant2".into(),
                Some(TypeDecl::Id(TypeId::Primitive(PrimitiveType::U32))),
            ),
        ]);
        let mut enum_def_visitor = EnumDefVisitor { variants: vec![] };

        accept_enum_def(&enum_def, &mut enum_def_visitor);

        assert!(itertools::equal(
            enum_def_visitor.variants,
            enum_def.variants()
        ));
    }
}
