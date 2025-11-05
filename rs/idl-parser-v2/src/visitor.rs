use super::ast::*;

// The main trait for visiting the entire IDL AST.
// Each method corresponds to a node in the AST.
// By default, each method calls the corresponding `accept_*` function
// to continue the traversal down the tree.
pub trait Visitor<'ast> {
    fn visit_idl_doc(&mut self, doc: &'ast IdlDoc) {
        accept_idl_doc(doc, self);
    }

    fn visit_program_unit(&mut self, program: &'ast ProgramUnit) {
        accept_program_unit(program, self);
    }

    fn visit_service_unit(&mut self, service: &'ast ServiceUnit) {
        accept_service_unit(service, self);
    }

    fn visit_ctor_func(&mut self, ctor_func: &'ast CtorFunc) {
        accept_ctor_func(ctor_func, self);
    }

    fn visit_program_service_item(&mut self, service_item: &'ast ProgramServiceItem) {
        accept_program_service_item(service_item, self);
    }

    fn visit_type(&mut self, ty: &'ast Type) {
        accept_type(ty, self);
    }

    fn visit_service_func(&mut self, service_func: &'ast ServiceFunc) {
        accept_service_func(service_func, self);
    }

    fn visit_service_event(&mut self, service_event: &'ast ServiceEvent) {
        accept_service_event(service_event, self);
    }

    fn visit_func_param(&mut self, func_param: &'ast FuncParam) {
        accept_func_param(func_param, self);
    }

    fn visit_type_decl(&mut self, type_decl: &'ast TypeDecl) {
        accept_type_decl(type_decl, self);
    }

    fn visit_type_parameter(&mut self, type_param: &'ast TypeParameter) {
        accept_type_parameter(type_param, self);
    }

    fn visit_type_def(&mut self, type_def: &'ast TypeDef) {
        accept_type_def(type_def, self);
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

    // TypeDecl variants
    fn visit_slice_type_decl(&mut self, item_type_decl: &'ast TypeDecl) {
        accept_type_decl(item_type_decl, self);
    }

    fn visit_array_type_decl(&mut self, item_type_decl: &'ast TypeDecl, _len: u32) {
        accept_type_decl(item_type_decl, self);
    }

    fn visit_tuple_type_decl(&mut self, items: &'ast Vec<TypeDecl>) {
        accept_tuple_type_decl(items, self);
    }

    fn visit_option_type_decl(&mut self, inner_type_decl: &'ast TypeDecl) {
        accept_type_decl(inner_type_decl, self);
    }

    fn visit_result_type_decl(
        &mut self,
        ok_type_decl: &'ast TypeDecl,
        err_type_decl: &'ast TypeDecl,
    ) {
        accept_type_decl(ok_type_decl, self);
        accept_type_decl(err_type_decl, self);
    }

    fn visit_primitive_type(&mut self, _primitive_type: PrimitiveType) {
        // Primitive types are leaf nodes, no further traversal needed.
    }

    fn visit_user_defined_type(&mut self, _path: &'ast str, generics: &'ast Vec<TypeDecl>) {
        accept_user_defined_type(generics, self);
    }
}

// Entry point for visiting the entire IDL document.
pub fn accept_idl_doc<'ast>(doc: &'ast IdlDoc, visitor: &mut (impl Visitor<'ast> + ?Sized)) {
    if let Some(program) = &doc.program {
        visitor.visit_program_unit(program);
    }
    for service in &doc.services {
        visitor.visit_service_unit(service);
    }
}

// Visits the children of a `ProgramUnit`.
pub fn accept_program_unit<'ast>(
    program: &'ast ProgramUnit,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    for ctor_func in &program.ctors {
        visitor.visit_ctor_func(ctor_func);
    }
    for service_item in &program.services {
        visitor.visit_program_service_item(service_item);
    }
    for ty in &program.types {
        visitor.visit_type(ty);
    }
}

// Visits the children of a `ServiceUnit`.
pub fn accept_service_unit<'ast>(
    service: &'ast ServiceUnit,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    for func in &service.funcs {
        visitor.visit_service_func(func);
    }
    for event in &service.events {
        visitor.visit_service_event(event);
    }
    for ty in &service.types {
        visitor.visit_type(ty);
    }
}

// Visits the children of a `CtorFunc`.
pub fn accept_ctor_func<'ast>(
    ctor_func: &'ast CtorFunc,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    for param in &ctor_func.params {
        visitor.visit_func_param(param);
    }
}

// Visits the children of a `ProgramServiceItem`.
pub fn accept_program_service_item<'ast>(
    _service_item: &'ast ProgramServiceItem,
    _visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    // This is a leaf node in terms of traversal, no children to visit.
}

// Visits the children of a `Type`.
pub fn accept_type<'ast>(ty: &'ast Type, visitor: &mut (impl Visitor<'ast> + ?Sized)) {
    for type_param in &ty.type_params {
        visitor.visit_type_parameter(type_param);
    }
    visitor.visit_type_def(&ty.def);
}

// Visits the children of a `ServiceFunc`.
pub fn accept_service_func<'ast>(
    service_func: &'ast ServiceFunc,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    for param in &service_func.params {
        visitor.visit_func_param(param);
    }
    visitor.visit_type_decl(&service_func.output);
    if let Some(throws_type) = &service_func.throws {
        visitor.visit_type_decl(throws_type);
    }
}

// Visits the children of a `ServiceEvent`.
pub fn accept_service_event<'ast>(
    service_event: &'ast ServiceEvent,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    // ServiceEvent is an alias for EnumVariant, so we visit its definition.
    visitor.visit_enum_variant(service_event);
}

// Visits the children of a `FuncParam`.
pub fn accept_func_param<'ast>(
    func_param: &'ast FuncParam,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    visitor.visit_type_decl(&func_param.type_decl);
}

// Visits the children of a `TypeDecl`.
pub fn accept_type_decl<'ast>(
    type_decl: &'ast TypeDecl,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    match type_decl {
        TypeDecl::Slice(item_type_decl) => {
            visitor.visit_slice_type_decl(item_type_decl);
        }
        TypeDecl::Array { item, len } => {
            visitor.visit_array_type_decl(item, *len);
        }
        TypeDecl::Tuple(items) => {
            visitor.visit_tuple_type_decl(items);
        }
        TypeDecl::Option(inner_type_decl) => {
            visitor.visit_option_type_decl(inner_type_decl);
        }
        TypeDecl::Result { ok, err } => {
            visitor.visit_result_type_decl(ok, err);
        }
        TypeDecl::Primitive(primitive_type) => {
            visitor.visit_primitive_type(*primitive_type);
        }
        TypeDecl::UserDefined { path, generics } => {
            visitor.visit_user_defined_type(path, generics);
        }
    }
}

// Visits the children of a `TypeParameter`.
pub fn accept_type_parameter<'ast>(
    type_param: &'ast TypeParameter,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    if let Some(ty) = &type_param.ty {
        visitor.visit_type_decl(ty);
    }
}

// Visits the children of a `TypeDef`.
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

// Visits the children of a `StructDef`.
pub fn accept_struct_def<'ast>(
    struct_def: &'ast StructDef,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    for field in &struct_def.fields {
        visitor.visit_struct_field(field);
    }
}

// Visits the children of a `StructField`.
pub fn accept_struct_field<'ast>(
    struct_field: &'ast StructField,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    visitor.visit_type_decl(&struct_field.type_decl);
}

// Visits the children of an `EnumDef`.
pub fn accept_enum_def<'ast>(enum_def: &'ast EnumDef, visitor: &mut (impl Visitor<'ast> + ?Sized)) {
    for variant in &enum_def.variants {
        visitor.visit_enum_variant(variant);
    }
}

// Visits the children of an `EnumVariant`.
pub fn accept_enum_variant<'ast>(
    enum_variant: &'ast EnumVariant,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    // EnumVariant contains a StructDef for its fields.
    visitor.visit_struct_def(&enum_variant.def);
}

// Visits the children of a `Tuple` TypeDecl.
pub fn accept_tuple_type_decl<'ast>(
    items: &'ast Vec<TypeDecl>,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    for item in items {
        visitor.visit_type_decl(item);
    }
}

// Visits the children of a `UserDefined` TypeDecl.
pub fn accept_user_defined_type<'ast>(
    generics: &'ast Vec<TypeDecl>,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    for generic in generics {
        visitor.visit_type_decl(generic);
    }
}
