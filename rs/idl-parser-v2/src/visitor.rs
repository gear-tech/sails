use crate::ast;

// The main trait for visiting the IDL AST.
//
// Each `visit_*` method corresponds to a node in the [ast].
// The default implementation of each method calls the corresponding `accept_*` function
// to continue the traversal down the tree. This allows a visitor to only override
// the methods for the nodes it is interested in.
pub trait Visitor<'ast> {
    /// Visits a program unit, [ast::ProgramUnit].
    fn visit_program_unit(&mut self, program: &'ast ast::ProgramUnit) {
        accept_program_unit(program, self);
    }

    /// Visits a service unit, [ast::ServiceUnit].
    fn visit_service_unit(&mut self, service: &'ast ast::ServiceUnit) {
        accept_service_unit(service, self);
    }

    /// Visits a constructor function, [ast::CtorFunc].
    fn visit_ctor_func(&mut self, ctor_func: &'ast ast::CtorFunc) {
        accept_ctor_func(ctor_func, self);
    }

    /// Visits a service export within a program, [ast::ServiceExpo].
    /// This is a leaf node.
    fn visit_service_expo(&mut self, service_expo: &'ast ast::ServiceExpo) {
        accept_service_expo(service_expo, self);
    }

    /// Visits a custom type definition, [ast::Type].
    fn visit_type(&mut self, ty: &'ast ast::Type) {
        accept_type(ty, self);
    }

    /// Visits a service function, [ast::ServiceFunc].
    fn visit_service_func(&mut self, service_func: &'ast ast::ServiceFunc) {
        accept_service_func(service_func, self);
    }

    /// Visits a service event, [ast::ServiceEvent].
    fn visit_service_event(&mut self, service_event: &'ast ast::ServiceEvent) {
        accept_service_event(service_event, self);
    }

    /// Visits a function parameter, [ast::FuncParam].
    fn visit_func_param(&mut self, func_param: &'ast ast::FuncParam) {
        accept_func_param(func_param, self);
    }

    /// Visits a type parameter for generics, [ast::TypeParameter].
    fn visit_type_parameter(&mut self, type_param: &'ast ast::TypeParameter) {
        accept_type_parameter(type_param, self);
    }

    /// Visits a type definition enum, [ast::TypeDef].
    fn visit_type_def(&mut self, type_def: &'ast ast::TypeDef) {
        accept_type_def(type_def, self);
    }

    /// Visits a struct definition, [ast::StructDef].
    fn visit_struct_def(&mut self, struct_def: &'ast ast::StructDef) {
        accept_struct_def(struct_def, self);
    }

    /// Visits a struct field, [ast::StructField].
    fn visit_struct_field(&mut self, struct_field: &'ast ast::StructField) {
        accept_struct_field(struct_field, self);
    }

    /// Visits an enum definition, [ast::EnumDef].
    fn visit_enum_def(&mut self, enum_def: &'ast ast::EnumDef) {
        accept_enum_def(enum_def, self);
    }

    /// Visits an enum variant, [ast::EnumVariant].
    fn visit_enum_variant(&mut self, enum_variant: &'ast ast::EnumVariant) {
        accept_enum_variant(enum_variant, self);
    }

    // ----- TypeDecl variants -----

    /// Visits a slice type declaration, `[T]`, from [ast::TypeDecl::Slice].
    fn visit_slice_type_decl(&mut self, item_type_decl: &'ast ast::TypeDecl) {
        accept_type_decl(item_type_decl, self);
    }

    /// Visits an array type declaration, `[T; N]`, from [ast::TypeDecl::Array].
    fn visit_array_type_decl(&mut self, item_type_decl: &'ast ast::TypeDecl, _len: u32) {
        accept_type_decl(item_type_decl, self);
    }

    /// Visits a tuple type declaration, `(T, U)`, from [ast::TypeDecl::Tuple].
    fn visit_tuple_type_decl(&mut self, items: &'ast [ast::TypeDecl]) {
        for item in items {
            accept_type_decl(item, self);
        }
    }

    /// Visits a primitive type, [ast::PrimitiveType].
    /// This is a leaf node.
    fn visit_primitive_type(&mut self, _primitive_type: ast::PrimitiveType) {
        // Primitive types are leaf nodes, no further traversal needed.
    }

    /// Visits a named type, `path::to::MyType<T>`, from [ast::TypeDecl::Named].
    fn visit_named_type_decl(&mut self, _name: &'ast str, generics: &'ast [ast::TypeDecl]) {
        for generic in generics {
            accept_type_decl(generic, self);
        }
    }
}

/// Traverses the children of an [ast::IdlDoc].
/// This is the main entry point for visiting the entire AST.
pub fn accept_idl_doc<'ast>(doc: &'ast ast::IdlDoc, visitor: &mut (impl Visitor<'ast> + ?Sized)) {
    if let Some(program) = &doc.program {
        visitor.visit_program_unit(program);
    }
    for service in &doc.services {
        visitor.visit_service_unit(service);
    }
}

/// Traverses the children of a [ast::ProgramUnit].
/// It visits constructors, services, and types within the program.
pub fn accept_program_unit<'ast>(
    program: &'ast ast::ProgramUnit,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    for ctor_func in &program.ctors {
        visitor.visit_ctor_func(ctor_func);
    }
    for service_item in &program.services {
        visitor.visit_service_expo(service_item);
    }
    for ty in &program.types {
        visitor.visit_type(ty);
    }
}

/// Traverses the children of a [ast::ServiceUnit].
/// It visits functions, events, and types within the service.
pub fn accept_service_unit<'ast>(
    service: &'ast ast::ServiceUnit,
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

/// Traverses the children of a [ast::CtorFunc].
/// It visits the function's parameters.
pub fn accept_ctor_func<'ast>(
    ctor_func: &'ast ast::CtorFunc,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    for param in &ctor_func.params {
        visitor.visit_func_param(param);
    }
}

/// Visits a [ast::ServiceExpo].
/// This is a leaf node in terms of traversal, so it does nothing.
pub fn accept_service_expo<'ast>(
    _service_expo: &'ast ast::ServiceExpo,
    _visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    // This is a leaf node, no children to visit.
}

/// Traverses the children of a [ast::Type].
/// It visits type parameters (generics) and the type definition.
pub fn accept_type<'ast>(ty: &'ast ast::Type, visitor: &mut (impl Visitor<'ast> + ?Sized)) {
    for type_param in &ty.type_params {
        visitor.visit_type_parameter(type_param);
    }
    visitor.visit_type_def(&ty.def);
}

/// Traverses the children of a [ast::ServiceFunc].
/// It visits parameters, the output type, and the optional throws type.
pub fn accept_service_func<'ast>(
    service_func: &'ast ast::ServiceFunc,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    for param in &service_func.params {
        visitor.visit_func_param(param);
    }
    accept_type_decl(&service_func.output, visitor);
    if let Some(throws_type) = &service_func.throws {
        accept_type_decl(throws_type, visitor);
    }
}

/// Traverses the children of a [ast::ServiceEvent].
/// A `ServiceEvent` is an alias for `EnumVariant`.
pub fn accept_service_event<'ast>(
    service_event: &'ast ast::ServiceEvent,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    // ServiceEvent is an alias for EnumVariant, so we visit its definition.
    visitor.visit_enum_variant(service_event);
}

/// Traverses the children of a [ast::FuncParam].
/// It visits the parameter's type declaration.
pub fn accept_func_param<'ast>(
    func_param: &'ast ast::FuncParam,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    accept_type_decl(&func_param.type_decl, visitor);
}

/// Traverses the children of a [ast::TypeDecl].
/// This function dispatches to the appropriate `visit_*` method based on the `TypeDecl` variant.
pub fn accept_type_decl<'ast>(
    type_decl: &'ast ast::TypeDecl,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    match type_decl {
        ast::TypeDecl::Slice(item_type_decl) => {
            visitor.visit_slice_type_decl(item_type_decl);
        }
        ast::TypeDecl::Array(item, len) => {
            visitor.visit_array_type_decl(item, *len);
        }
        ast::TypeDecl::Tuple(items) => {
            visitor.visit_tuple_type_decl(items);
        }
        ast::TypeDecl::Primitive(primitive_type) => {
            visitor.visit_primitive_type(*primitive_type);
        }
        ast::TypeDecl::Named(name, generics) => {
            visitor.visit_named_type_decl(name, generics);
        }
    }
}

/// Traverses the children of a [ast::TypeParameter].
/// It visits the concrete type if it is specified.
pub fn accept_type_parameter<'ast>(
    type_param: &'ast ast::TypeParameter,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    if let Some(ty) = &type_param.ty {
        accept_type_decl(ty, visitor);
    }
}

/// Traverses the children of a [ast::TypeDef].
/// This function dispatches to the appropriate `visit_*` method for `Struct` or `Enum`.
pub fn accept_type_def<'ast>(
    type_def: &'ast ast::TypeDef,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    match type_def {
        ast::TypeDef::Struct(struct_def) => {
            visitor.visit_struct_def(struct_def);
        }
        ast::TypeDef::Enum(enum_def) => {
            visitor.visit_enum_def(enum_def);
        }
    }
}

/// Traverses the children of a [ast::StructDef].
/// It visits all fields of the struct.
pub fn accept_struct_def<'ast>(
    struct_def: &'ast ast::StructDef,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    for field in &struct_def.fields {
        visitor.visit_struct_field(field);
    }
}

/// Traverses the children of a [ast::StructField].
/// It visits the field's type declaration.
pub fn accept_struct_field<'ast>(
    struct_field: &'ast ast::StructField,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    accept_type_decl(&struct_field.type_decl, visitor);
}

/// Traverses the children of an [ast::EnumDef].
/// It visits all variants of the enum.
pub fn accept_enum_def<'ast>(
    enum_def: &'ast ast::EnumDef,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    for variant in &enum_def.variants {
        visitor.visit_enum_variant(variant);
    }
}

/// Traverses the children of an [ast::EnumVariant].
/// It visits the struct definition of the variant's fields.
pub fn accept_enum_variant<'ast>(
    enum_variant: &'ast ast::EnumVariant,
    visitor: &mut (impl Visitor<'ast> + ?Sized),
) {
    // EnumVariant contains a StructDef for its fields.
    visitor.visit_struct_def(&enum_variant.def);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_idl;

    #[test]
    fn test_visitor_traversal() {
        const IDL_SOURCE: &str = include_str!("../tests/idls/demo.idl");

        #[derive(Default)]
        struct CountingVisitor {
            program_count: u32,
            service_count: u32,
        }

        impl<'ast> Visitor<'ast> for CountingVisitor {
            fn visit_program_unit(&mut self, program: &'ast ast::ProgramUnit) {
                self.program_count += 1;
                accept_program_unit(program, self);
            }

            fn visit_service_unit(&mut self, service: &'ast ast::ServiceUnit) {
                self.service_count += 1;
                accept_service_unit(service, self);
            }
        }

        let doc = parse_idl(IDL_SOURCE).expect("Failed to parse IDL");

        let mut visitor = CountingVisitor::default();
        accept_idl_doc(&doc, &mut visitor);

        assert_eq!(visitor.program_count, 1);
        assert_eq!(visitor.service_count, 6);
    }

    #[test]
    fn test_visitor_traversal_test_idl() {
        const IDL_SOURCE: &str = include_str!("../tests/idls/test.idl");

        #[derive(Default)]
        struct CountingVisitor {
            program_count: u32,
            service_count: u32,
        }

        impl<'ast> Visitor<'ast> for CountingVisitor {
            fn visit_program_unit(&mut self, program: &'ast ast::ProgramUnit) {
                self.program_count += 1;
                accept_program_unit(program, self);
            }

            fn visit_service_unit(&mut self, service: &'ast ast::ServiceUnit) {
                self.service_count += 1;
                accept_service_unit(service, self);
            }
        }

        let doc = parse_idl(IDL_SOURCE).expect("Failed to parse IDL");

        let mut visitor = CountingVisitor::default();
        accept_idl_doc(&doc, &mut visitor);

        assert_eq!(visitor.program_count, 0);
        assert_eq!(visitor.service_count, 2);
    }
}
