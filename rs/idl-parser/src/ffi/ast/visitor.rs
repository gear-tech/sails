use super::*;
use crate::{ast as raw_ast, ast::visitor as raw_visitor, ast::visitor::Visitor as RawVisitor};
use std::ptr;
use wrapper::VisitorWrapper;

#[repr(C, packed)]
pub struct Visitor {
    visit_ctor: unsafe extern "C" fn(context: *const (), *const Ctor),
    visit_service: unsafe extern "C" fn(context: *const (), *const Service),
    visit_type: unsafe extern "C" fn(context: *const (), *const Type),
    visit_vector_type_decl: unsafe extern "C" fn(context: *const (), *const TypeDecl),
    visit_array_type_decl: unsafe extern "C" fn(context: *const (), *const TypeDecl, u32),
    visit_map_type_decl: unsafe extern "C" fn(context: *const (), *const TypeDecl, *const TypeDecl),
    visit_optional_type_decl: unsafe extern "C" fn(context: *const (), *const TypeDecl),
    visit_result_type_decl:
        unsafe extern "C" fn(context: *const (), *const TypeDecl, *const TypeDecl),
    visit_primitive_type_id: unsafe extern "C" fn(context: *const (), PrimitiveType),
    visit_user_defined_type_id: unsafe extern "C" fn(context: *const (), *const u8, u32),
    visit_ctor_func: unsafe extern "C" fn(context: *const (), *const CtorFunc),
    visit_service_func: unsafe extern "C" fn(context: *const (), *const ServiceFunc),
    visit_service_event: unsafe extern "C" fn(context: *const (), *const ServiceEvent),
    visit_func_param: unsafe extern "C" fn(context: *const (), *const FuncParam),
    visit_func_output: unsafe extern "C" fn(context: *const (), *const TypeDecl),
    visit_struct_def: unsafe extern "C" fn(context: *const (), *const StructDef),
    visit_struct_field: unsafe extern "C" fn(context: *const (), *const StructField),
    visit_enum_def: unsafe extern "C" fn(context: *const (), *const EnumDef),
    visit_enum_variant: unsafe extern "C" fn(context: *const (), *const EnumVariant),
}

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn visit_ctor(context: *const (), ctor: *const Ctor);
    fn visit_service(context: *const (), service: *const Service);
    fn visit_type(context: *const (), r#type: *const Type);
    fn visit_vector_type_decl(context: *const (), item_type_decl: *const TypeDecl);
    fn visit_array_type_decl(context: *const (), item_type_decl: *const TypeDecl, len: u32);
    fn visit_map_type_decl(
        context: *const (),
        key_type_decl: *const TypeDecl,
        value_type_decl: *const TypeDecl,
    );
    fn visit_optional_type_decl(context: *const (), optional_type_decl: *const TypeDecl);
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
    fn visit_ctor_func(context: *const (), func: *const CtorFunc);
    fn visit_service_func(context: *const (), func: *const ServiceFunc);
    fn visit_service_event(context: *const (), event: *const ServiceEvent);
    fn visit_func_param(context: *const (), func_param: *const FuncParam);
    fn visit_func_output(context: *const (), func_output: *const TypeDecl);
    fn visit_struct_def(context: *const (), struct_def: *const StructDef);
    fn visit_struct_field(context: *const (), struct_field: *const StructField);
    fn visit_enum_def(context: *const (), enum_def: *const EnumDef);
    fn visit_enum_variant(context: *const (), enum_variant: *const EnumVariant);
}

#[cfg(target_arch = "wasm32")]
static VISITOR: Visitor = Visitor {
    visit_ctor,
    visit_service,
    visit_type,
    visit_vector_type_decl,
    visit_array_type_decl,
    visit_map_type_decl,
    visit_optional_type_decl,
    visit_result_type_decl,
    visit_primitive_type_id,
    visit_user_defined_type_id,
    visit_ctor_func,
    visit_service_func,
    visit_service_event,
    visit_func_param,
    visit_func_output,
    visit_struct_def,
    visit_struct_field,
    visit_enum_def,
    visit_enum_variant,
};

macro_rules! deref {
    ($expr:expr) => {{
        if $expr.is_null() {
            return ErrorCode::NullPtr;
        }
        unsafe { $expr.as_ref() }.unwrap()
    }};
}

macro_rules! deref_visitor {
    ($context:expr, $visitor:expr) => {
        match VisitorWrapper::new($context, $visitor) {
            Ok(visitor) => visitor,
            Err(err) => return err,
        }
    };
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
extern "C" fn accept_program(program: *const Program, context: *const ()) -> ErrorCode {
    accept_program_impl(program, context, &VISITOR)
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
extern "C" fn accept_program(
    program: *const Program,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    accept_program_impl(program, context, visitor)
}

fn accept_program_impl(
    program: *const Program,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    let program = deref!(program);
    let mut visitor = deref_visitor!(context, visitor);
    raw_visitor::accept_program(program, &mut visitor);
    ErrorCode::Ok
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
extern "C" fn accept_ctor(ctor: *const Ctor, context: *const ()) -> ErrorCode {
    accept_ctor_impl(ctor, context, &VISITOR)
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
extern "C" fn accept_ctor(
    ctor: *const Ctor,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    accept_ctor_impl(ctor, context, visitor)
}

fn accept_ctor_impl(ctor: *const Ctor, context: *const (), visitor: *const Visitor) -> ErrorCode {
    let ctor = deref!(ctor);
    let mut visitor = deref_visitor!(context, visitor);
    raw_visitor::accept_ctor(ctor.raw_ptr.as_ref(), &mut visitor);
    ErrorCode::Ok
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
extern "C" fn accept_ctor_func(func: *const CtorFunc, context: *const ()) -> ErrorCode {
    accept_ctor_func_impl(func, context, &VISITOR)
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
extern "C" fn accept_ctor_func(
    func: *const CtorFunc,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    accept_ctor_func_impl(func, context, visitor)
}

fn accept_ctor_func_impl(
    func: *const CtorFunc,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    let func = deref!(func);
    let mut visitor = deref_visitor!(context, visitor);
    raw_visitor::accept_ctor_func(func.raw_ptr.as_ref(), &mut visitor);
    ErrorCode::Ok
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
extern "C" fn accept_service(service: *const Service, context: *const ()) -> ErrorCode {
    accept_service_impl(service, context, &VISITOR)
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
extern "C" fn accept_service(
    service: *const Service,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    accept_service_impl(service, context, visitor)
}

fn accept_service_impl(
    service: *const Service,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    let service = deref!(service);
    let mut visitor = deref_visitor!(context, visitor);
    raw_visitor::accept_service(service.raw_ptr.as_ref(), &mut visitor);
    ErrorCode::Ok
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
extern "C" fn accept_service_func(func: *const ServiceFunc, context: *const ()) -> ErrorCode {
    accept_service_func_impl(func, context, &VISITOR)
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
extern "C" fn accept_service_func(
    func: *const ServiceFunc,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    accept_service_func_impl(func, context, visitor)
}

fn accept_service_func_impl(
    func: *const ServiceFunc,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    let func = deref!(func);
    let mut visitor = deref_visitor!(context, visitor);
    raw_visitor::accept_service_func(func.raw_ptr.as_ref(), &mut visitor);
    ErrorCode::Ok
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
extern "C" fn accept_service_event(event: *const ServiceEvent, context: *const ()) -> ErrorCode {
    accept_service_event_impl(event, context, &VISITOR)
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
extern "C" fn accept_service_event(
    event: *const ServiceEvent,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    accept_service_event_impl(event, context, visitor)
}

fn accept_service_event_impl(
    event: *const ServiceEvent,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    let event = deref!(event);
    let mut visitor = deref_visitor!(context, visitor);
    raw_visitor::accept_service_event(event.raw_ptr.as_ref(), &mut visitor);
    ErrorCode::Ok
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
extern "C" fn accept_func_param(func_param: *const FuncParam, context: *const ()) -> ErrorCode {
    accept_func_param_impl(func_param, context, &VISITOR)
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
extern "C" fn accept_func_param(
    func_param: *const FuncParam,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    accept_func_param_impl(func_param, context, visitor)
}

fn accept_func_param_impl(
    func_param: *const FuncParam,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    let func_param = deref!(func_param);
    let mut visitor = deref_visitor!(context, visitor);
    raw_visitor::accept_func_param(func_param.raw_ptr.as_ref(), &mut visitor);
    ErrorCode::Ok
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
extern "C" fn accept_type(r#type: *const Type, context: *const ()) -> ErrorCode {
    accept_type_impl(r#type, context, &VISITOR)
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
extern "C" fn accept_type(
    r#type: *const Type,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    accept_type_impl(r#type, context, visitor)
}

fn accept_type_impl(r#type: *const Type, context: *const (), visitor: *const Visitor) -> ErrorCode {
    let r#type = deref!(r#type);
    let mut visitor = deref_visitor!(context, visitor);
    raw_visitor::accept_type(r#type.raw_ptr.as_ref(), &mut visitor);
    ErrorCode::Ok
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
extern "C" fn accept_type_decl(type_decl: *const TypeDecl, context: *const ()) -> ErrorCode {
    accept_type_decl_impl(type_decl, context, &VISITOR)
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
extern "C" fn accept_type_decl(
    type_decl: *const TypeDecl,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    accept_type_decl_impl(type_decl, context, visitor)
}

fn accept_type_decl_impl(
    type_decl: *const TypeDecl,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    let type_decl = deref!(type_decl);
    let mut visitor = deref_visitor!(context, visitor);
    raw_visitor::accept_type_decl(type_decl.raw_ptr.as_ref(), &mut visitor);
    ErrorCode::Ok
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
extern "C" fn accept_struct_def(struct_def: *const StructDef, context: *const ()) -> ErrorCode {
    accept_struct_def_impl(struct_def, context, &VISITOR)
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
extern "C" fn accept_struct_def(
    struct_def: *const StructDef,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    accept_struct_def_impl(struct_def, context, visitor)
}

fn accept_struct_def_impl(
    struct_def: *const StructDef,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    let struct_def = deref!(struct_def);
    let mut visitor = deref_visitor!(context, visitor);
    raw_visitor::accept_struct_def(struct_def.raw_ptr.as_ref(), &mut visitor);
    ErrorCode::Ok
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
extern "C" fn accept_struct_field(
    struct_field: *const StructField,
    context: *const (),
) -> ErrorCode {
    accept_struct_field_impl(struct_field, context, &VISITOR)
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
extern "C" fn accept_struct_field(
    struct_field: *const StructField,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    accept_struct_field_impl(struct_field, context, visitor)
}

fn accept_struct_field_impl(
    struct_field: *const StructField,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    let struct_field = deref!(struct_field);
    let mut visitor = deref_visitor!(context, visitor);
    raw_visitor::accept_struct_field(struct_field.raw_ptr.as_ref(), &mut visitor);
    ErrorCode::Ok
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
extern "C" fn accept_enum_def(enum_def: *const EnumDef, context: *const ()) -> ErrorCode {
    accept_enum_def_impl(enum_def, context, &VISITOR)
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
extern "C" fn accept_enum_def(
    enum_def: *const EnumDef,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    accept_enum_def_impl(enum_def, context, visitor)
}

fn accept_enum_def_impl(
    enum_def: *const EnumDef,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    let enum_def = deref!(enum_def);
    let mut visitor = deref_visitor!(context, visitor);
    raw_visitor::accept_enum_def(enum_def.raw_ptr.as_ref(), &mut visitor);
    ErrorCode::Ok
}

#[cfg(target_arch = "wasm32")]
#[no_mangle]
extern "C" fn accept_enum_variant(
    enum_variant: *const EnumVariant,
    context: *const (),
) -> ErrorCode {
    accept_enum_variant_impl(enum_variant, context, &VISITOR)
}

#[cfg(not(target_arch = "wasm32"))]
#[no_mangle]
extern "C" fn accept_enum_variant(
    enum_variant: *const EnumVariant,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    accept_enum_variant_impl(enum_variant, context, visitor)
}

fn accept_enum_variant_impl(
    enum_variant: *const EnumVariant,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    let enum_variant = deref!(enum_variant);
    let mut visitor = deref_visitor!(context, visitor);
    raw_visitor::accept_enum_variant(enum_variant.raw_ptr.as_ref(), &mut visitor);
    ErrorCode::Ok
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

    impl VisitorWrapper<'_> {
        pub fn new(context: *const (), visitor: *const Visitor) -> Result<Self, ErrorCode> {
            if visitor.is_null() {
                return Err(ErrorCode::NullPtr);
            }

            Ok(Self {
                context,
                visitor: unsafe { visitor.as_ref() }.unwrap(),
            })
        }
    }

    impl<'ast> RawVisitor<'ast> for VisitorWrapper<'_> {
        fn visit_ctor(&mut self, ctor: &'ast raw_ast::Ctor) {
            if fn_ptr_addr!(self.visitor.visit_ctor).is_null() {
                return raw_visitor::accept_ctor(ctor, self);
            }
            let ctor = Ctor {
                raw_ptr: ctor.into(),
            };
            unsafe { (self.visitor.visit_ctor)(self.context, &ctor) };
        }

        fn visit_service(&mut self, service: &'ast raw_ast::Service) {
            if fn_ptr_addr!(self.visitor.visit_service).is_null() {
                return raw_visitor::accept_service(service, self);
            }
            let name_bytes = service.name().as_bytes();
            let service = Service {
                raw_ptr: service.into(),
                name_ptr: name_bytes.as_ptr(),
                name_len: name_bytes.len() as u32,
            };
            unsafe { (self.visitor.visit_service)(self.context, &service) };
        }

        fn visit_type(&mut self, r#type: &'ast raw_ast::Type) {
            if fn_ptr_addr!(self.visitor.visit_type).is_null() {
                return raw_visitor::accept_type(r#type, self);
            }
            let name_bytes = r#type.name().as_bytes();
            let docs = r#type.docs().join("\n");
            let docs_bytes = docs.as_bytes();

            let r#type = Type {
                name_ptr: name_bytes.as_ptr(),
                name_len: name_bytes.len() as u32,
                raw_ptr: r#type.into(),
                docs_ptr: docs_bytes.as_ptr(),
                docs_len: docs_bytes.len() as u32,
            };
            unsafe { (self.visitor.visit_type)(self.context, &r#type) };
        }

        fn visit_vector_type_decl(&mut self, item_type_decl: &'ast raw_ast::TypeDecl) {
            if fn_ptr_addr!(self.visitor.visit_vector_type_decl).is_null() {
                return raw_visitor::accept_type_decl(item_type_decl, self);
            }
            let item_type_decl = TypeDecl {
                raw_ptr: item_type_decl.into(),
            };
            unsafe { (self.visitor.visit_vector_type_decl)(self.context, &item_type_decl) };
        }

        fn visit_array_type_decl(&mut self, item_type_decl: &'ast ast::TypeDecl, len: u32) {
            if fn_ptr_addr!(self.visitor.visit_array_type_decl).is_null() {
                return raw_visitor::accept_type_decl(item_type_decl, self);
            }
            let item_type_decl = TypeDecl {
                raw_ptr: item_type_decl.into(),
            };
            unsafe { (self.visitor.visit_array_type_decl)(self.context, &item_type_decl, len) };
        }

        fn visit_map_type_decl(
            &mut self,
            key_type_decl: &'ast ast::TypeDecl,
            value_type_decl: &'ast ast::TypeDecl,
        ) {
            if fn_ptr_addr!(self.visitor.visit_map_type_decl).is_null() {
                raw_visitor::accept_type_decl(key_type_decl, self);
                raw_visitor::accept_type_decl(value_type_decl, self);
                return;
            }
            let key_type_decl = TypeDecl {
                raw_ptr: key_type_decl.into(),
            };
            let value_type_decl = TypeDecl {
                raw_ptr: value_type_decl.into(),
            };
            unsafe {
                (self.visitor.visit_map_type_decl)(self.context, &key_type_decl, &value_type_decl)
            };
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

        fn visit_result_type_decl(
            &mut self,
            ok_type_decl: &'ast raw_ast::TypeDecl,
            err_type_decl: &'ast raw_ast::TypeDecl,
        ) {
            if fn_ptr_addr!(self.visitor.visit_result_type_decl).is_null() {
                raw_visitor::accept_type_decl(ok_type_decl, self);
                raw_visitor::accept_type_decl(err_type_decl, self);
                return;
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

        fn visit_primitive_type_id(&mut self, primitive_type_id: raw_ast::PrimitiveType) {
            if fn_ptr_addr!(self.visitor.visit_primitive_type_id).is_null() {
                return;
            }
            unsafe { (self.visitor.visit_primitive_type_id)(self.context, primitive_type_id) };
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

        fn visit_ctor_func(&mut self, func: &'ast raw_ast::CtorFunc) {
            if fn_ptr_addr!(self.visitor.visit_ctor_func).is_null() {
                return raw_visitor::accept_ctor_func(func, self);
            }
            let func_name_bytes = func.name().as_bytes();
            let docs = func.docs().join("\n");
            let docs_bytes = docs.as_bytes();

            let func = CtorFunc {
                name_ptr: func_name_bytes.as_ptr(),
                name_len: func_name_bytes.len() as u32,
                raw_ptr: func.into(),
                docs_ptr: docs_bytes.as_ptr(),
                docs_len: docs_bytes.len() as u32,
            };
            unsafe { (self.visitor.visit_ctor_func)(self.context, &func) };
        }

        fn visit_service_func(&mut self, func: &'ast raw_ast::ServiceFunc) {
            if fn_ptr_addr!(self.visitor.visit_service_func).is_null() {
                return raw_visitor::accept_service_func(func, self);
            }
            let func_name_bytes = func.name().as_bytes();
            let docs = func.docs().join("\n");
            let docs_bytes = docs.as_bytes();

            let func = ServiceFunc {
                name_ptr: func_name_bytes.as_ptr(),
                name_len: func_name_bytes.len() as u32,
                is_query: func.is_query(),
                raw_ptr: func.into(),
                docs_ptr: docs_bytes.as_ptr(),
                docs_len: docs_bytes.len() as u32,
            };
            unsafe { (self.visitor.visit_service_func)(self.context, &func) };
        }

        fn visit_service_event(&mut self, event: &'ast ast::ServiceEvent) {
            if fn_ptr_addr!(self.visitor.visit_service_event).is_null() {
                return raw_visitor::accept_service_event(event, self);
            }
            let event_name_bytes = event.name().as_bytes();
            let docs = event.docs().join("\n");
            let docs_bytes = docs.as_bytes();

            let event = ServiceEvent {
                name_ptr: event_name_bytes.as_ptr(),
                name_len: event_name_bytes.len() as u32,
                raw_ptr: event.into(),
                docs_ptr: docs_bytes.as_ptr(),
                docs_len: docs_bytes.len() as u32,
            };
            unsafe { (self.visitor.visit_service_event)(self.context, &event) };
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
            let docs = struct_field.docs().join("\n");
            let docs_bytes = docs.as_bytes();

            let struct_field = StructField {
                name_ptr: struct_field_name_bytes.map_or(ptr::null(), |n| n.as_ptr()),
                name_len: struct_field_name_bytes.map_or(0, |n| n.len() as u32),
                raw_ptr: struct_field.into(),
                docs_ptr: docs_bytes.as_ptr(),
                docs_len: docs_bytes.len() as u32,
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
            let docs = enum_variant.docs().join("\n");
            let docs_bytes = docs.as_bytes();

            let enum_variant = EnumVariant {
                name_ptr: enum_variant_name_bytes.as_ptr(),
                name_len: enum_variant_name_bytes.len() as u32,
                raw_ptr: enum_variant.into(),
                docs_ptr: docs_bytes.as_ptr(),
                docs_len: docs_bytes.len() as u32,
            };
            unsafe { (self.visitor.visit_enum_variant)(self.context, &enum_variant) };
        }
    }
}
