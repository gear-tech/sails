use sails_idl_parser_v2::visitor::{Visitor, accept_idl_doc};
use sails_idl_parser_v2::{ast, parse_idl};

#[derive(Default, Debug)]
struct CountingVisitor {
    program_unit: u32,
    service_unit: u32,
    ctor_func: u32,
    func_param: u32,
    ty: u32,
    slice_type_decl: u32,
    array_type_decl: u32,
    tuple_type_decl: u32,
    named_type_decl: u32,
    primitive_type: u32,
    service_func: u32,
    service_event: u32,
    struct_def: u32,
    struct_field: u32,
    enum_def: u32,
    enum_variant: u32,
    service_expo: u32,
    type_parameter: u32,
    type_def: u32,
}

impl<'ast> Visitor<'ast> for CountingVisitor {
    fn visit_program_unit(&mut self, node: &'ast ast::ProgramUnit) {
        self.program_unit += 1;
        sails_idl_parser_v2::visitor::accept_program_unit(node, self);
    }

    fn visit_service_unit(&mut self, node: &'ast ast::ServiceUnit) {
        self.service_unit += 1;
        sails_idl_parser_v2::visitor::accept_service_unit(node, self);
    }

    fn visit_ctor_func(&mut self, node: &'ast ast::CtorFunc) {
        self.ctor_func += 1;
        sails_idl_parser_v2::visitor::accept_ctor_func(node, self);
    }

    fn visit_func_param(&mut self, node: &'ast ast::FuncParam) {
        self.func_param += 1;
        sails_idl_parser_v2::visitor::accept_func_param(node, self);
    }

    fn visit_type(&mut self, node: &'ast ast::Type) {
        self.ty += 1;
        sails_idl_parser_v2::visitor::accept_type(node, self);
    }

    fn visit_slice_type_decl(&mut self, inner: &'ast ast::TypeDecl) {
        self.slice_type_decl += 1;
        sails_idl_parser_v2::visitor::accept_type_decl(inner, self);
    }

    fn visit_array_type_decl(&mut self, inner: &'ast ast::TypeDecl, _len: u32) {
        self.array_type_decl += 1;
        sails_idl_parser_v2::visitor::accept_type_decl(inner, self);
    }

    fn visit_tuple_type_decl(&mut self, items: &'ast [ast::TypeDecl]) {
        self.tuple_type_decl += 1;
        for item in items {
            sails_idl_parser_v2::visitor::accept_type_decl(item, self);
        }
    }

    fn visit_named_type_decl(&mut self, _name: &'ast str, generics: &'ast [ast::TypeDecl]) {
        self.named_type_decl += 1;
        for generic in generics {
            sails_idl_parser_v2::visitor::accept_type_decl(generic, self);
        }
    }

    fn visit_primitive_type(&mut self, _t: ast::PrimitiveType) {
        self.primitive_type += 1;
    }

    fn visit_service_func(&mut self, node: &'ast ast::ServiceFunc) {
        self.service_func += 1;
        sails_idl_parser_v2::visitor::accept_service_func(node, self);
    }

    fn visit_service_event(&mut self, node: &'ast ast::ServiceEvent) {
        self.service_event += 1;
        sails_idl_parser_v2::visitor::accept_service_event(node, self);
    }

    fn visit_struct_def(&mut self, node: &'ast ast::StructDef) {
        self.struct_def += 1;
        sails_idl_parser_v2::visitor::accept_struct_def(node, self);
    }

    fn visit_struct_field(&mut self, node: &'ast ast::StructField) {
        self.struct_field += 1;
        sails_idl_parser_v2::visitor::accept_struct_field(node, self);
    }

    fn visit_enum_def(&mut self, node: &'ast ast::EnumDef) {
        self.enum_def += 1;
        sails_idl_parser_v2::visitor::accept_enum_def(node, self);
    }

    fn visit_enum_variant(&mut self, node: &'ast ast::EnumVariant) {
        self.enum_variant += 1;
        sails_idl_parser_v2::visitor::accept_enum_variant(node, self);
    }

    fn visit_service_expo(&mut self, node: &'ast ast::ServiceExpo) {
        self.service_expo += 1;
        sails_idl_parser_v2::visitor::accept_service_expo(node, self);
    }

    fn visit_type_parameter(&mut self, node: &'ast ast::TypeParameter) {
        self.type_parameter += 1;
        sails_idl_parser_v2::visitor::accept_type_parameter(node, self);
    }

    fn visit_type_def(&mut self, node: &'ast ast::TypeDef) {
        self.type_def += 1;
        sails_idl_parser_v2::visitor::accept_type_def(node, self);
    }
}

#[test]
fn test_full_coverage_rust_visitor() {
    const IDL_SOURCE: &str = include_str!("idls/full_coverage.idl");
    let doc = parse_idl(IDL_SOURCE).expect("Failed to parse IDL");

    let mut visitor = CountingVisitor::default();
    accept_idl_doc(&doc, &mut visitor);

    println!("{doc:#?}");

    assert_eq!(visitor.program_unit, 1);
    assert_eq!(visitor.service_unit, 2);
    assert_eq!(visitor.ctor_func, 1);
    assert_eq!(visitor.func_param, 1);
    assert_eq!(visitor.ty, 6);
    assert_eq!(visitor.slice_type_decl, 1);
    assert_eq!(visitor.array_type_decl, 1);
    assert_eq!(visitor.tuple_type_decl, 1);
    assert_eq!(visitor.named_type_decl, 5);
    assert_eq!(visitor.primitive_type, 22);
    assert_eq!(visitor.service_func, 3);
    assert_eq!(visitor.service_event, 3);
    assert_eq!(visitor.struct_def, 11);
    assert_eq!(visitor.struct_field, 17);
    assert_eq!(visitor.enum_def, 1);
    assert_eq!(visitor.enum_variant, 6);
    assert_eq!(visitor.service_expo, 2);
    assert_eq!(visitor.type_parameter, 1);
    assert_eq!(visitor.type_def, 6);

    let total_type_decls = visitor.slice_type_decl
        + visitor.array_type_decl
        + visitor.tuple_type_decl
        + visitor.named_type_decl
        + visitor.primitive_type;

    assert_eq!(total_type_decls, 30, "Total TypeDecl nodes visited");
}
