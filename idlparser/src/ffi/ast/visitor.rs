use super::*;
use crate::{ast as raw_ast, ast::visitor as raw_visitor, ast::visitor::Visitor as RawVisitor};
use std::ptr;
use wrapper::VisitorWrapper;

#[repr(C, packed)]
pub struct Visitor {
    visit_service: unsafe extern "C" fn(context: *const (), *const Service),
    visit_type: unsafe extern "C" fn(context: *const (), *const Type),
    visit_optional_type_decl: unsafe extern "C" fn(context: *const (), *const TypeDecl),
    visit_vector_type_decl: unsafe extern "C" fn(context: *const (), *const TypeDecl),
    visit_result_type_decl:
        unsafe extern "C" fn(context: *const (), *const TypeDecl, *const TypeDecl),
    visit_primitive_type_id: unsafe extern "C" fn(context: *const (), PrimitiveType),
    visit_user_defined_type_id: unsafe extern "C" fn(context: *const (), *const u8, u32),
    visit_func: unsafe extern "C" fn(context: *const (), *const Func),
    visit_func_param: unsafe extern "C" fn(context: *const (), *const FuncParam),
    visit_func_output: unsafe extern "C" fn(context: *const (), *const TypeDecl),
    visit_struct_def: unsafe extern "C" fn(context: *const (), *const StructDef),
    visit_struct_field: unsafe extern "C" fn(context: *const (), *const StructField),
    visit_enum_def: unsafe extern "C" fn(context: *const (), *const EnumDef),
    visit_enum_variant: unsafe extern "C" fn(context: *const (), *const EnumVariant),
}

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn visit_service(context: *const (), service: *const Service);
    fn visit_type(context: *const (), r#type: *const Type);
    fn visit_optional_type_decl(context: *const (), optional_type_decl: *const TypeDecl);
    fn visit_vector_type_decl(context: *const (), vector_type_decl: *const TypeDecl);
    fn visit_result_type_decl(
        context: *const (),
        ok_type_decl: *const TypeDecl,
        err_type_decl: *const TypeDecl,
    );
    fn visit_primitive_type_id(context: *const (), primitive_type_id: PrimitiveType);
    fn visit_user_defined_type_id(
        context: *const (),
        user_defined_type_id_ptr: *const u8,
        user_defined_type_id_len: u32,
    );
    fn visit_func(context: *const (), func: *const Func);
    fn visit_func_param(context: *const (), func_param: *const FuncParam);
    fn visit_func_output(context: *const (), func_output: *const TypeDecl);
    fn visit_struct_def(context: *const (), struct_def: *const StructDef);
    fn visit_struct_field(context: *const (), struct_field: *const StructField);
    fn visit_enum_def(context: *const (), enum_def: *const EnumDef);
    fn visit_enum_variant(context: *const (), enum_variant: *const EnumVariant);
}

#[cfg(target_arch = "wasm32")]
static VISITOR: Visitor = Visitor {
    visit_service,
    visit_type,
    visit_optional_type_decl,
    visit_vector_type_decl,
    visit_result_type_decl,
    visit_primitive_type_id,
    visit_user_defined_type_id,
    visit_func,
    visit_func_param,
    visit_func_output,
    visit_struct_def,
    visit_struct_field,
    visit_enum_def,
    visit_enum_variant,
};

#[cfg(target_arch = "wasm32")]
#[no_mangle]
extern "C" fn accept_program(program: *const Program, context: *const ()) {
    accept_program_impl(program, context, &VISITOR)
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
extern "C" fn accept_program(program: *const Program, context: *const (), visitor: *const Visitor) {
    accept_program_impl(program, context, visitor)
}

fn accept_program_impl(program: *const Program, context: *const (), visitor: *const Visitor) {
    let program = unsafe { program.as_ref() }.unwrap();
    let mut visitor = VisitorWrapper::new(context, visitor);
    raw_visitor::accept_program(program, &mut visitor);
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
extern "C" fn accept_service(service: *const Service, context: *const ()) {
    accept_service_impl(service, context, &VISITOR)
}
#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
extern "C" fn accept_service(service: *const Service, context: *const (), visitor: *const Visitor) {
    accept_service_impl(service, context, visitor)
}

fn accept_service_impl(service: *const Service, context: *const (), visitor: *const Visitor) {
    let service = unsafe { service.as_ref() }.unwrap();
    let mut visitor = VisitorWrapper::new(context, visitor);
    raw_visitor::accept_service(service.raw_ptr.as_ref(), &mut visitor);
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
extern "C" fn accept_func(func: *const Func, context: *const ()) {
    accept_func_impl(func, context, &VISITOR)
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
extern "C" fn accept_func(func: *const Func, context: *const (), visitor: *const Visitor) {
    accept_func_impl(func, context, visitor)
}

fn accept_func_impl(func: *const Func, context: *const (), visitor: *const Visitor) {
    let func = unsafe { func.as_ref() }.unwrap();
    let mut visitor = VisitorWrapper::new(context, visitor);
    raw_visitor::accept_func(func.raw_ptr.as_ref(), &mut visitor);
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
extern "C" fn accept_func_param(func_param: *const FuncParam, context: *const ()) {
    accept_func_param_impl(func_param, context, &VISITOR)
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
extern "C" fn accept_func_param(
    func_param: *const FuncParam,
    context: *const (),
    visitor: *const Visitor,
) {
    accept_func_param_impl(func_param, context, visitor)
}

fn accept_func_param_impl(
    func_param: *const FuncParam,
    context: *const (),
    visitor: *const Visitor,
) {
    let func_param = unsafe { func_param.as_ref() }.unwrap();
    let mut visitor = VisitorWrapper::new(context, visitor);
    raw_visitor::accept_func_param(func_param.raw_ptr.as_ref(), &mut visitor);
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
extern "C" fn accept_type(r#type: *const Type, context: *const ()) {
    accept_type_impl(r#type, context, &VISITOR)
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
extern "C" fn accept_type(r#type: *const Type, context: *const (), visitor: *const Visitor) {
    accept_type_impl(r#type, context, visitor)
}

fn accept_type_impl(r#type: *const Type, context: *const (), visitor: *const Visitor) {
    let r#type = unsafe { r#type.as_ref() }.unwrap();
    let mut visitor = VisitorWrapper::new(context, visitor);
    raw_visitor::accept_type(r#type.raw_ptr.as_ref(), &mut visitor);
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
extern "C" fn accept_type_decl(type_decl: *const TypeDecl, context: *const ()) {
    accept_type_decl_impl(type_decl, context, &VISITOR)
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
extern "C" fn accept_type_decl(
    type_decl: *const TypeDecl,
    context: *const (),
    visitor: *const Visitor,
) {
    accept_type_decl_impl(type_decl, context, visitor)
}

fn accept_type_decl_impl(type_decl: *const TypeDecl, context: *const (), visitor: *const Visitor) {
    let type_decl = unsafe { type_decl.as_ref() }.unwrap();
    let mut visitor = VisitorWrapper::new(context, visitor);
    raw_visitor::accept_type_decl(type_decl.raw_ptr.as_ref(), &mut visitor);
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
extern "C" fn accept_struct_def(struct_def: *const StructDef, context: *const ()) {
    accept_struct_def_impl(struct_def, context, &VISITOR)
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
extern "C" fn accept_struct_def(
    struct_def: *const StructDef,
    context: *const (),
    visitor: *const Visitor,
) {
    accept_struct_def_impl(struct_def, context, visitor)
}

fn accept_struct_def_impl(
    struct_def: *const StructDef,
    context: *const (),
    visitor: *const Visitor,
) {
    let struct_def = unsafe { struct_def.as_ref() }.unwrap();
    let mut visitor = VisitorWrapper::new(context, visitor);
    raw_visitor::accept_struct_def(struct_def.raw_ptr.as_ref(), &mut visitor);
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
extern "C" fn accept_struct_field(struct_field: *const StructField, context: *const ()) {
    accept_struct_field_impl(struct_field, context, &VISITOR)
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
extern "C" fn accept_struct_field(
    struct_field: *const StructField,
    context: *const (),
    visitor: *const Visitor,
) {
    accept_struct_field_impl(struct_field, context, visitor)
}

fn accept_struct_field_impl(
    struct_field: *const StructField,
    context: *const (),
    visitor: *const Visitor,
) {
    let struct_field = unsafe { struct_field.as_ref() }.unwrap();
    let mut visitor = VisitorWrapper::new(context, visitor);
    raw_visitor::accept_struct_field(struct_field.raw_ptr.as_ref(), &mut visitor);
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
extern "C" fn accept_enum_def(enum_def: *const EnumDef, context: *const ()) {
    accept_enum_def_impl(enum_def, context, &VISITOR)
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
extern "C" fn accept_enum_def(
    enum_def: *const EnumDef,
    context: *const (),
    visitor: *const Visitor,
) {
    accept_enum_def_impl(enum_def, context, visitor)
}

fn accept_enum_def_impl(enum_def: *const EnumDef, context: *const (), visitor: *const Visitor) {
    let enum_def = unsafe { enum_def.as_ref() }.unwrap();
    let mut visitor = VisitorWrapper::new(context, visitor);
    raw_visitor::accept_enum_def(enum_def.raw_ptr.as_ref(), &mut visitor);
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
extern "C" fn accept_enum_variant(enum_variant: *const EnumVariant, context: *const ()) {
    accept_enum_variant_impl(enum_variant, context, &VISITOR)
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
extern "C" fn accept_enum_variant(
    enum_variant: *const EnumVariant,
    context: *const (),
    visitor: *const Visitor,
) {
    accept_enum_variant_impl(enum_variant, context, visitor)
}

fn accept_enum_variant_impl(
    enum_variant: *const EnumVariant,
    context: *const (),
    visitor: *const Visitor,
) {
    let enum_variant = unsafe { enum_variant.as_ref() }.unwrap();
    let mut visitor = VisitorWrapper::new(context, visitor);
    raw_visitor::accept_enum_variant(enum_variant.raw_ptr.as_ref(), &mut visitor);
}

mod wrapper {
    use super::*;

    macro_rules! fn_ptr_addr {
        ($fn_ptr: expr) => {{
            let fn_ptr_addr = $fn_ptr as *const ();
            fn_ptr_addr
        }};
    }

    pub(super) struct VisitorWrapper<'a> {
        context: *const (),
        visitor: &'a Visitor,
    }

    impl<'a> VisitorWrapper<'a> {
        pub fn new(context: *const (), visitor: *const Visitor) -> Self {
            Self {
                context,
                visitor: unsafe { visitor.as_ref() }.unwrap(),
            }
        }
    }

    impl<'a, 'ast> RawVisitor<'ast> for VisitorWrapper<'a> {
        fn visit_service(&mut self, service: &'ast raw_ast::Service) {
            if fn_ptr_addr!(self.visitor.visit_service).is_null() {
                return raw_visitor::accept_service(service, self);
            }
            let service = Service {
                raw_ptr: service.into(),
            };
            unsafe { (self.visitor.visit_service)(self.context, &service) };
        }

        fn visit_type(&mut self, r#type: &'ast raw_ast::Type) {
            if fn_ptr_addr!(self.visitor.visit_type).is_null() {
                return raw_visitor::accept_type(r#type, self);
            }
            let name_bytes = r#type.name().as_bytes();
            let r#type = Type {
                name_ptr: name_bytes.as_ptr(),
                name_len: name_bytes.len() as u32,
                raw_ptr: r#type.into(),
            };
            unsafe { (self.visitor.visit_type)(self.context, &r#type) };
        }

        fn visit_optional_type_decl(&mut self, optional_type_decl: &'ast raw_ast::TypeDecl) {
            if fn_ptr_addr!(self.visitor.visit_optional_type_decl).is_null() {
                return raw_visitor::accept_type_decl(optional_type_decl, self);
            }
            let optional_type_decl = TypeDecl {
                raw_ptr: optional_type_decl.into(),
            };
            unsafe { (self.visitor.visit_optional_type_decl)(self.context, &optional_type_decl) };
        }

        fn visit_vector_type_decl(&mut self, vector_type_decl: &'ast raw_ast::TypeDecl) {
            if fn_ptr_addr!(self.visitor.visit_vector_type_decl).is_null() {
                return raw_visitor::accept_type_decl(vector_type_decl, self);
            }
            let vector_type_decl = TypeDecl {
                raw_ptr: vector_type_decl.into(),
            };
            unsafe { (self.visitor.visit_vector_type_decl)(self.context, &vector_type_decl) };
        }

        fn visit_result_type_decl(
            &mut self,
            ok_type_decl: &'ast raw_ast::TypeDecl,
            err_type_decl: &'ast raw_ast::TypeDecl,
        ) {
            if fn_ptr_addr!(self.visitor.visit_result_type_decl).is_null() {
                return raw_visitor::accept_type_decl(ok_type_decl, self);
            }
            let ok_type_decl = TypeDecl {
                raw_ptr: ok_type_decl.into(),
            };
            let err_type_decl = TypeDecl {
                raw_ptr: err_type_decl.into(),
            };
            unsafe {
                (self.visitor.visit_result_type_decl)(self.context, &ok_type_decl, &err_type_decl)
            };
        }

        fn visit_primitive_type_id(&mut self, primitive_type_id: &'ast raw_ast::PrimitiveType) {
            if fn_ptr_addr!(self.visitor.visit_primitive_type_id).is_null() {
                return;
            }
            unsafe { (self.visitor.visit_primitive_type_id)(self.context, *primitive_type_id) };
        }

        fn visit_user_defined_type_id(&mut self, user_defined_type_id: &'ast str) {
            if fn_ptr_addr!(self.visitor.visit_user_defined_type_id).is_null() {
                return;
            }
            let user_defined_type_id = user_defined_type_id.as_bytes();
            unsafe {
                (self.visitor.visit_user_defined_type_id)(
                    self.context,
                    user_defined_type_id.as_ptr(),
                    user_defined_type_id.len() as u32,
                )
            };
        }

        fn visit_func(&mut self, func: &'ast raw_ast::Func) {
            if fn_ptr_addr!(self.visitor.visit_func).is_null() {
                return raw_visitor::accept_func(func, self);
            }
            let func_name_bytes = func.name().as_bytes();
            let func = Func {
                name_ptr: func_name_bytes.as_ptr(),
                name_len: func_name_bytes.len() as u32,
                is_query: func.is_query(),
                raw_ptr: func.into(),
            };
            unsafe { (self.visitor.visit_func)(self.context, &func) };
        }

        fn visit_func_param(&mut self, func_param: &'ast raw_ast::FuncParam) {
            if fn_ptr_addr!(self.visitor.visit_func_param).is_null() {
                return raw_visitor::accept_func_param(func_param, self);
            }
            let func_param_name_bytes = func_param.name().as_bytes();
            let func_param = FuncParam {
                name_ptr: func_param_name_bytes.as_ptr(),
                name_len: func_param_name_bytes.len() as u32,
                raw_ptr: func_param.into(),
            };
            unsafe { (self.visitor.visit_func_param)(self.context, &func_param) };
        }

        fn visit_func_output(&mut self, func_output: &'ast raw_ast::TypeDecl) {
            if fn_ptr_addr!(self.visitor.visit_func_output).is_null() {
                return raw_visitor::accept_type_decl(func_output, self);
            }
            let func_output = TypeDecl {
                raw_ptr: func_output.into(),
            };
            unsafe { (self.visitor.visit_func_output)(self.context, &func_output) };
        }

        fn visit_struct_def(&mut self, struct_def: &'ast raw_ast::StructDef) {
            if fn_ptr_addr!(self.visitor.visit_struct_def).is_null() {
                return raw_visitor::accept_struct_def(struct_def, self);
            }
            let struct_def = StructDef {
                raw_ptr: struct_def.into(),
            };
            unsafe { (self.visitor.visit_struct_def)(self.context, &struct_def) };
        }

        fn visit_struct_field(&mut self, struct_field: &'ast raw_ast::StructField) {
            if fn_ptr_addr!(self.visitor.visit_struct_field).is_null() {
                return raw_visitor::accept_struct_field(struct_field, self);
            }
            let struct_field_name_bytes = struct_field.name().map(|n| n.as_bytes());
            let struct_field = StructField {
                name_ptr: struct_field_name_bytes.map_or(ptr::null(), |n| n.as_ptr()),
                name_len: struct_field_name_bytes.map_or(0, |n| n.len() as u32),
                raw_ptr: struct_field.into(),
            };
            unsafe { (self.visitor.visit_struct_field)(self.context, &struct_field) };
        }

        fn visit_enum_def(&mut self, enum_def: &'ast raw_ast::EnumDef) {
            if fn_ptr_addr!(self.visitor.visit_enum_def).is_null() {
                return raw_visitor::accept_enum_def(enum_def, self);
            }
            let enum_def = EnumDef {
                raw_ptr: enum_def.into(),
            };
            unsafe { (self.visitor.visit_enum_def)(self.context, &enum_def) };
        }

        fn visit_enum_variant(&mut self, enum_variant: &'ast raw_ast::EnumVariant) {
            if fn_ptr_addr!(self.visitor.visit_enum_variant).is_null() {
                return raw_visitor::accept_enum_variant(enum_variant, self);
            }
            let enum_variant_name_bytes = enum_variant.name().as_bytes();
            let enum_variant = EnumVariant {
                name_ptr: enum_variant_name_bytes.as_ptr(),
                name_len: enum_variant_name_bytes.len() as u32,
                raw_ptr: enum_variant.into(),
            };
            unsafe { (self.visitor.visit_enum_variant)(self.context, &enum_variant) };
        }
    }
}
