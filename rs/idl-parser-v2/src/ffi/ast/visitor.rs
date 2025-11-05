use super::{
    Allocations, CtorFunc, EnumDef, EnumVariant, ErrorCode, FFIString, FuncParam, IdlDoc,
    ProgramServiceItem, ProgramUnit, ServiceEvent, ServiceFunc, ServiceUnit, StructDef,
    StructField, Type, TypeDecl, TypeDef, TypeParameter,
};
use crate::ast::{self, visitor::Visitor as RawVisitor};
use paste::paste;

#[repr(C)]
pub struct Visitor {
    pub visit_program_unit:
        Option<unsafe extern "C" fn(context: *const (), program: *const ProgramUnit)>,
    pub visit_service_unit:
        Option<unsafe extern "C" fn(context: *const (), service: *const ServiceUnit)>,
    pub visit_ctor_func: Option<unsafe extern "C" fn(context: *const (), ctor: *const CtorFunc)>,
    pub visit_func_param: Option<unsafe extern "C" fn(context: *const (), param: *const FuncParam)>,
    pub visit_type: Option<unsafe extern "C" fn(context: *const (), ty: *const Type)>,
    pub visit_slice_type_decl:
        Option<unsafe extern "C" fn(context: *const (), item_ty: *const TypeDecl)>,
    pub visit_array_type_decl:
        Option<unsafe extern "C" fn(context: *const (), item_ty: *const TypeDecl, len: u32)>,
    pub visit_tuple_type_decl: Option<
        unsafe extern "C" fn(context: *const (), items_ptr: *const TypeDecl, items_len: u32),
    >,
    pub visit_option_type_decl:
        Option<unsafe extern "C" fn(context: *const (), inner_ty: *const TypeDecl)>,
    pub visit_result_type_decl: Option<
        unsafe extern "C" fn(context: *const (), ok_ty: *const TypeDecl, err_ty: *const TypeDecl),
    >,
    pub visit_primitive_type:
        Option<unsafe extern "C" fn(context: *const (), primitive: ast::PrimitiveType)>,
    pub visit_user_defined_type:
        Option<unsafe extern "C" fn(context: *const (), path_ptr: *const u8, path_len: u32)>,
    pub visit_service_func:
        Option<unsafe extern "C" fn(context: *const (), func: *const ServiceFunc)>,
    pub visit_service_event:
        Option<unsafe extern "C" fn(context: *const (), event: *const ServiceEvent)>,
    pub visit_struct_def: Option<unsafe extern "C" fn(context: *const (), def: *const StructDef)>,
    pub visit_struct_field:
        Option<unsafe extern "C" fn(context: *const (), field: *const StructField)>,
    pub visit_enum_def: Option<unsafe extern "C" fn(context: *const (), def: *const EnumDef)>,
    pub visit_enum_variant:
        Option<unsafe extern "C" fn(context: *const (), variant: *const EnumVariant)>,
    pub visit_program_service_item:
        Option<unsafe extern "C" fn(context: *const (), service_item: *const ProgramServiceItem)>,
    pub visit_type_decl:
        Option<unsafe extern "C" fn(context: *const (), type_decl: *const TypeDecl)>,
    pub visit_type_parameter:
        Option<unsafe extern "C" fn(context: *const (), type_param: *const TypeParameter)>,
    pub visit_type_def: Option<unsafe extern "C" fn(context: *const (), type_def: *const TypeDef)>,
}

#[cfg(target_arch = "wasm32")]
extern "C" {
    fn visit_program_unit(context: *const (), program: *const ProgramUnit);
    fn visit_service_unit(context: *const (), service: *const ServiceUnit);
    fn visit_ctor_func(context: *const (), ctor: *const CtorFunc);
    fn visit_func_param(context: *const (), param: *const FuncParam);
    fn visit_type(context: *const (), ty: *const Type);
    fn visit_slice_type_decl(context: *const (), item_ty: *const TypeDecl);
    fn visit_array_type_decl(context: *const (), item_ty: *const TypeDecl, len: u32);
    fn visit_tuple_type_decl(context: *const (), items_ptr: *const TypeDecl, items_len: u32);
    fn visit_option_type_decl(context: *const (), inner_ty: *const TypeDecl);
    fn visit_result_type_decl(context: *const (), ok_ty: *const TypeDecl, err_ty: *const TypeDecl);
    fn visit_primitive_type(context: *const (), primitive: ast::PrimitiveType);
    fn visit_user_defined_type(context: *const (), path_ptr: *const u8, path_len: u32);
    fn visit_service_func(context: *const (), func: *const ServiceFunc);
    fn visit_service_event(context: *const (), event: *const ServiceEvent);
    fn visit_struct_def(context: *const (), def: *const StructDef);
    fn visit_struct_field(context: *const (), field: *const StructField);
    fn visit_enum_def(context: *const (), def: *const EnumDef);
    fn visit_enum_variant(context: *const (), variant: *const EnumVariant);
    fn visit_program_service_item(context: *const (), service_item: *const ProgramServiceItem);
    fn visit_type_decl(context: *const (), type_decl: *const TypeDecl);
    fn visit_type_parameter(context: *const (), type_param: *const TypeParameter);
    fn visit_type_def(context: *const (), type_def: *const TypeDef);
}

#[cfg(target_arch = "wasm32")]
static VISITOR: Visitor = Visitor {
    visit_program_unit: Some(visit_program_unit),
    visit_service_unit: Some(visit_service_unit),
    visit_ctor_func: Some(visit_ctor_func),
    visit_func_param: Some(visit_func_param),
    visit_type: Some(visit_type),
    visit_slice_type_decl: Some(visit_slice_type_decl),
    visit_array_type_decl: Some(visit_array_type_decl),
    visit_tuple_type_decl: Some(visit_tuple_type_decl),
    visit_option_type_decl: Some(visit_option_type_decl),
    visit_result_type_decl: Some(visit_result_type_decl),
    visit_primitive_type: Some(visit_primitive_type),
    visit_user_defined_type: Some(visit_user_defined_type),
    visit_service_func: Some(visit_service_func),
    visit_service_event: Some(visit_service_event),
    visit_struct_def: Some(visit_struct_def),
    visit_struct_field: Some(visit_struct_field),
    visit_enum_def: Some(visit_enum_def),
    visit_enum_variant: Some(visit_enum_variant),
    visit_program_service_item: Some(visit_program_service_item),
    visit_type_decl: Some(visit_type_decl),
    visit_type_parameter: Some(visit_type_parameter),
    visit_type_def: Some(visit_type_def),
};

struct VisitorWrapper<'a> {
    context: *const (),
    visitor: &'a Visitor,
    allocations: Allocations,
}

impl<'a> VisitorWrapper<'a> {
    fn new(context: *const (), visitor: &'a Visitor) -> Self {
        Self {
            context,
            visitor,
            allocations: Allocations::new(),
        }
    }
}

impl<'a, 'ast> RawVisitor<'ast> for VisitorWrapper<'a> {
    fn visit_idl_doc(&mut self, doc: &'ast ast::IdlDoc) {
        crate::ast::visitor::accept_idl_doc(doc, self);
    }

    fn visit_program_unit(&mut self, program: &'ast ast::ProgramUnit) {
        if let Some(visit) = self.visitor.visit_program_unit {
            let ffi_program = ProgramUnit::from_ast(program, &mut self.allocations);
            unsafe { visit(self.context, &ffi_program) };
            return;
        }
        crate::ast::visitor::accept_program_unit(program, self);
    }

    fn visit_service_unit(&mut self, service: &'ast ast::ServiceUnit) {
        if let Some(visit) = self.visitor.visit_service_unit {
            let ffi_service = ServiceUnit::from_ast(service, &mut self.allocations);
            unsafe { visit(self.context, &ffi_service) };
            return;
        }
        crate::ast::visitor::accept_service_unit(service, self);
    }

    fn visit_ctor_func(&mut self, ctor: &'ast ast::CtorFunc) {
        if let Some(visit) = self.visitor.visit_ctor_func {
            let ffi_ctor = CtorFunc::from_ast(ctor, &mut self.allocations);
            unsafe { visit(self.context, &ffi_ctor) };
            return;
        }
        crate::ast::visitor::accept_ctor_func(ctor, self);
    }

    fn visit_func_param(&mut self, param: &'ast ast::FuncParam) {
        if let Some(visit) = self.visitor.visit_func_param {
            let ffi_param = FuncParam::from_ast(param, &mut self.allocations);
            unsafe { visit(self.context, &ffi_param) };
            return;
        }
        crate::ast::visitor::accept_func_param(param, self);
    }

    fn visit_type(&mut self, ty: &'ast ast::Type) {
        if let Some(visit) = self.visitor.visit_type {
            let ffi_ty = Type::from_ast(ty, &mut self.allocations);
            unsafe { visit(self.context, &ffi_ty) };
            return;
        }
        crate::ast::visitor::accept_type(ty, self);
    }

    fn visit_slice_type_decl(&mut self, item_type_decl: &'ast ast::TypeDecl) {
        if let Some(visit) = self.visitor.visit_slice_type_decl {
            let ffi_item_ty = TypeDecl::from_ast(item_type_decl, &mut self.allocations);
            unsafe { visit(self.context, &ffi_item_ty) };
            return;
        }
        crate::ast::visitor::accept_type_decl(item_type_decl, self);
    }

    fn visit_array_type_decl(&mut self, item_type_decl: &'ast ast::TypeDecl, len: u32) {
        if let Some(visit) = self.visitor.visit_array_type_decl {
            let ffi_item_ty = TypeDecl::from_ast(item_type_decl, &mut self.allocations);
            unsafe { visit(self.context, &ffi_item_ty, len) };
            return;
        }
        crate::ast::visitor::accept_type_decl(item_type_decl, self);
    }

    fn visit_tuple_type_decl(&mut self, items: &'ast Vec<ast::TypeDecl>) {
        if let Some(visit) = self.visitor.visit_tuple_type_decl {
            let ffi_items: Vec<TypeDecl> = items
                .iter()
                .map(|item| TypeDecl::from_ast(item, &mut self.allocations))
                .collect();
            let boxed_slice = ffi_items.into_boxed_slice();
            let ptr = boxed_slice.as_ptr();
            let len = boxed_slice.len() as u32;
            self.allocations.type_decls.push(boxed_slice);
            unsafe { visit(self.context, ptr, len) };
            return;
        }
        crate::ast::visitor::accept_tuple_type_decl(items, self);
    }

    fn visit_option_type_decl(&mut self, inner_type_decl: &'ast ast::TypeDecl) {
        if let Some(visit) = self.visitor.visit_option_type_decl {
            let ffi_inner_ty = TypeDecl::from_ast(inner_type_decl, &mut self.allocations);
            unsafe { visit(self.context, &ffi_inner_ty) };
            return;
        }
        crate::ast::visitor::accept_type_decl(inner_type_decl, self);
    }

    fn visit_result_type_decl(
        &mut self,
        ok_type_decl: &'ast ast::TypeDecl,
        err_type_decl: &'ast ast::TypeDecl,
    ) {
        if let Some(visit) = self.visitor.visit_result_type_decl {
            let ffi_ok_ty = TypeDecl::from_ast(ok_type_decl, &mut self.allocations);
            let ffi_err_ty = TypeDecl::from_ast(err_type_decl, &mut self.allocations);
            unsafe { visit(self.context, &ffi_ok_ty, &ffi_err_ty) };
            return;
        }
        crate::ast::visitor::accept_type_decl(ok_type_decl, self);
        crate::ast::visitor::accept_type_decl(err_type_decl, self);
    }

    fn visit_primitive_type(&mut self, primitive_type: ast::PrimitiveType) {
        if let Some(visit) = self.visitor.visit_primitive_type {
            unsafe { visit(self.context, primitive_type) };
        }
    }

    fn visit_user_defined_type(&mut self, path: &'ast str, generics: &'ast Vec<ast::TypeDecl>) {
        if let Some(visit) = self.visitor.visit_user_defined_type {
            let path_ffi = FFIString {
                ptr: path.as_ptr(),
                len: path.len() as u32,
            };
            unsafe { visit(self.context, path_ffi.ptr, path_ffi.len) };
            return;
        }
        crate::ast::visitor::accept_user_defined_type(generics, self);
    }

    fn visit_service_func(&mut self, func: &'ast ast::ServiceFunc) {
        if let Some(visit) = self.visitor.visit_service_func {
            let ffi_func = ServiceFunc::from_ast(func, &mut self.allocations);
            unsafe { visit(self.context, &ffi_func) };
            return;
        }
        crate::ast::visitor::accept_service_func(func, self);
    }

    fn visit_service_event(&mut self, event: &'ast ast::ServiceEvent) {
        if let Some(visit) = self.visitor.visit_service_event {
            let ffi_event = ServiceEvent::from_ast(event, &mut self.allocations);
            unsafe { visit(self.context, &ffi_event) };
            return;
        }
        crate::ast::visitor::accept_service_event(event, self);
    }

    fn visit_struct_def(&mut self, def: &'ast ast::StructDef) {
        if let Some(visit) = self.visitor.visit_struct_def {
            let ffi_def = StructDef::from_ast(def, &mut self.allocations);
            unsafe { visit(self.context, &ffi_def) };
            return;
        }
        crate::ast::visitor::accept_struct_def(def, self);
    }

    fn visit_struct_field(&mut self, field: &'ast ast::StructField) {
        if let Some(visit) = self.visitor.visit_struct_field {
            let ffi_field = StructField::from_ast(field, &mut self.allocations);
            unsafe { visit(self.context, &ffi_field) };
            return;
        }
        crate::ast::visitor::accept_struct_field(field, self);
    }

    fn visit_enum_def(&mut self, def: &'ast ast::EnumDef) {
        if let Some(visit) = self.visitor.visit_enum_def {
            let ffi_def = EnumDef::from_ast(def, &mut self.allocations);
            unsafe { visit(self.context, &ffi_def) };
            return;
        }
        crate::ast::visitor::accept_enum_def(def, self);
    }

    fn visit_enum_variant(&mut self, variant: &'ast ast::EnumVariant) {
        if let Some(visit) = self.visitor.visit_enum_variant {
            let ffi_variant = EnumVariant::from_ast(variant, &mut self.allocations);
            unsafe { visit(self.context, &ffi_variant) };
            return;
        }
        crate::ast::visitor::accept_enum_variant(variant, self);
    }

    fn visit_program_service_item(&mut self, service_item: &'ast ast::ProgramServiceItem) {
        if let Some(visit) = self.visitor.visit_program_service_item {
            let ffi_item = ProgramServiceItem::from_ast(service_item, &mut self.allocations);
            unsafe { visit(self.context, &ffi_item) };
            return;
        }
        crate::ast::visitor::accept_program_service_item(service_item, self);
    }

    fn visit_type_decl(&mut self, type_decl: &'ast ast::TypeDecl) {
        if let Some(visit) = self.visitor.visit_type_decl {
            let ffi_type_decl = TypeDecl::from_ast(type_decl, &mut self.allocations);
            unsafe { visit(self.context, &ffi_type_decl) };
            return;
        }
        crate::ast::visitor::accept_type_decl(type_decl, self);
    }

    fn visit_type_parameter(&mut self, type_param: &'ast ast::TypeParameter) {
        if let Some(visit) = self.visitor.visit_type_parameter {
            let ffi_param = TypeParameter::from_ast(type_param, &mut self.allocations);
            unsafe { visit(self.context, &ffi_param) };
            return;
        }
        crate::ast::visitor::accept_type_parameter(type_param, self);
    }

    fn visit_type_def(&mut self, type_def: &'ast ast::TypeDef) {
        if let Some(visit) = self.visitor.visit_type_def {
            let ffi_type_def = TypeDef::from_ast(type_def, &mut self.allocations);
            unsafe { visit(self.context, &ffi_type_def) };
            return;
        }
        crate::ast::visitor::accept_type_def(type_def, self);
    }
}

// Manually defined accept_* functions for TypeDecl variants, PrimitiveType, and UserDefinedType

fn accept_slice_type_decl_impl(
    type_decl: *const TypeDecl,
    context: *const (),
    visitor: &Visitor,
) -> ErrorCode {
    if type_decl.is_null() {
        return ErrorCode::NullPtr;
    }
    let mut wrapper = VisitorWrapper::new(context, visitor);
    let raw_node: &ast::TypeDecl = unsafe { (*type_decl).raw_ptr.as_ref() };
    if let ast::TypeDecl::Slice(item_type_decl) = raw_node {
        wrapper.visit_slice_type_decl(item_type_decl);
    } else {
        return ErrorCode::NullPtr;
    }
    ErrorCode::Ok
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accept_slice_type_decl(
    type_decl: *const TypeDecl,
    context: *const (),
) -> ErrorCode {
    accept_slice_type_decl_impl(type_decl, context, &VISITOR)
}

/// Traverses the children of a slice type declaration.
///
/// # Safety
///
/// - `type_decl` must be a valid pointer to a `TypeDecl` representing a slice.
/// - `visitor` must be a valid pointer to a `Visitor` struct.
#[cfg(not(target_arch = "wasm32"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accept_slice_type_decl(
    type_decl: *const TypeDecl,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    if visitor.is_null() {
        return ErrorCode::NullPtr;
    }
    accept_slice_type_decl_impl(type_decl, context, unsafe { &*visitor })
}

fn accept_array_type_decl_impl(
    type_decl: *const TypeDecl,
    context: *const (),
    visitor: &Visitor,
) -> ErrorCode {
    if type_decl.is_null() {
        return ErrorCode::NullPtr;
    }
    let mut wrapper = VisitorWrapper::new(context, visitor);
    let raw_node: &ast::TypeDecl = unsafe { (*type_decl).raw_ptr.as_ref() };
    if let ast::TypeDecl::Array { item, len } = raw_node {
        wrapper.visit_array_type_decl(item, *len);
    } else {
        return ErrorCode::NullPtr;
    }
    ErrorCode::Ok
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accept_array_type_decl(
    type_decl: *const TypeDecl,

    context: *const (),
) -> ErrorCode {
    accept_array_type_decl_impl(type_decl, context, &VISITOR)
}

/// Traverses the children of an array type declaration.
///
/// # Safety
///
/// - `type_decl` must be a valid pointer to a `TypeDecl` representing an array.
/// - `visitor` must be a valid pointer to a `Visitor` struct.
#[cfg(not(target_arch = "wasm32"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accept_array_type_decl(
    type_decl: *const TypeDecl,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    if visitor.is_null() {
        return ErrorCode::NullPtr;
    }
    accept_array_type_decl_impl(type_decl, context, unsafe { &*visitor })
}

fn accept_tuple_type_decl_impl(
    type_decl: *const TypeDecl,
    context: *const (),
    visitor: &Visitor,
) -> ErrorCode {
    if type_decl.is_null() {
        return ErrorCode::NullPtr;
    }
    let mut wrapper = VisitorWrapper::new(context, visitor);
    let raw_node: &ast::TypeDecl = unsafe { (*type_decl).raw_ptr.as_ref() };
    if let ast::TypeDecl::Tuple(items) = raw_node {
        wrapper.visit_tuple_type_decl(items);
    } else {
        return ErrorCode::NullPtr;
    }
    ErrorCode::Ok
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accept_tuple_type_decl(
    type_decl: *const TypeDecl,

    context: *const (),
) -> ErrorCode {
    accept_tuple_type_decl_impl(type_decl, context, &VISITOR)
}

/// Traverses the children of a tuple type declaration.
///
/// # Safety
///
/// - `type_decl` must be a valid pointer to a `TypeDecl` representing a tuple.
/// - `visitor` must be a valid pointer to a `Visitor` struct.
#[cfg(not(target_arch = "wasm32"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accept_tuple_type_decl(
    type_decl: *const TypeDecl,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    if visitor.is_null() {
        return ErrorCode::NullPtr;
    }
    accept_tuple_type_decl_impl(type_decl, context, unsafe { &*visitor })
}

fn accept_option_type_decl_impl(
    type_decl: *const TypeDecl,
    context: *const (),
    visitor: &Visitor,
) -> ErrorCode {
    if type_decl.is_null() {
        return ErrorCode::NullPtr;
    }
    let mut wrapper = VisitorWrapper::new(context, visitor);
    let raw_node: &ast::TypeDecl = unsafe { (*type_decl).raw_ptr.as_ref() };
    if let ast::TypeDecl::Option(inner_type_decl) = raw_node {
        wrapper.visit_option_type_decl(inner_type_decl);
    } else {
        return ErrorCode::NullPtr;
    }
    ErrorCode::Ok
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]

pub unsafe extern "C" fn accept_option_type_decl(
    type_decl: *const TypeDecl,
    context: *const (),
) -> ErrorCode {
    accept_option_type_decl_impl(type_decl, context, &VISITOR)
}

/// Traverses the children of an option type declaration.
///
/// # Safety
///
/// - `type_decl` must be a valid pointer to a `TypeDecl` representing an option.
/// - `visitor` must be a valid pointer to a `Visitor` struct.
#[cfg(not(target_arch = "wasm32"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accept_option_type_decl(
    type_decl: *const TypeDecl,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    if visitor.is_null() {
        return ErrorCode::NullPtr;
    }
    accept_option_type_decl_impl(type_decl, context, unsafe { &*visitor })
}

fn accept_result_type_decl_impl(
    type_decl: *const TypeDecl,
    context: *const (),
    visitor: &Visitor,
) -> ErrorCode {
    if type_decl.is_null() {
        return ErrorCode::NullPtr;
    }
    let mut wrapper = VisitorWrapper::new(context, visitor);
    let raw_node: &ast::TypeDecl = unsafe { (*type_decl).raw_ptr.as_ref() };
    if let ast::TypeDecl::Result { ok, err } = raw_node {
        wrapper.visit_result_type_decl(ok, err);
    } else {
        return ErrorCode::NullPtr;
    }
    ErrorCode::Ok
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accept_result_type_decl(
    type_decl: *const TypeDecl,
    context: *const (),
) -> ErrorCode {
    accept_result_type_decl_impl(type_decl, context, &VISITOR)
}

/// Traverses the children of a result type declaration.
///
/// # Safety
///
/// - `type_decl` must be a valid pointer to a `TypeDecl` representing a result.
/// - `visitor` must be a valid pointer to a `Visitor` struct.
#[cfg(not(target_arch = "wasm32"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accept_result_type_decl(
    type_decl: *const TypeDecl,

    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    if visitor.is_null() {
        return ErrorCode::NullPtr;
    }

    accept_result_type_decl_impl(type_decl, context, unsafe { &*visitor })
}

fn accept_primitive_type_impl(
    primitive_type: ast::PrimitiveType,
    context: *const (),
    visitor: &Visitor,
) -> ErrorCode {
    let mut wrapper = VisitorWrapper::new(context, visitor);
    wrapper.visit_primitive_type(primitive_type);
    ErrorCode::Ok
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accept_primitive_type(
    primitive_type: ast::PrimitiveType,
    context: *const (),
) -> ErrorCode {
    accept_primitive_type_impl(primitive_type, context, &VISITOR)
}

/// Visits a primitive type.
///
/// # Safety
///
/// - `visitor` must be a valid pointer to a `Visitor` struct.
#[cfg(not(target_arch = "wasm32"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accept_primitive_type(
    primitive_type: ast::PrimitiveType,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    if visitor.is_null() {
        return ErrorCode::NullPtr;
    }
    accept_primitive_type_impl(primitive_type, context, unsafe { &*visitor })
}

fn accept_user_defined_type_impl(
    type_decl: *const TypeDecl,
    context: *const (),
    visitor: &Visitor,
) -> ErrorCode {
    if type_decl.is_null() {
        return ErrorCode::NullPtr;
    }
    let mut wrapper = VisitorWrapper::new(context, visitor);
    let raw_node: &ast::TypeDecl = unsafe { (*type_decl).raw_ptr.as_ref() };
    if let ast::TypeDecl::UserDefined { path, generics } = raw_node {
        wrapper.visit_user_defined_type(path, generics);
    } else {
        return ErrorCode::NullPtr;
    }
    ErrorCode::Ok
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accept_user_defined_type(
    type_decl: *const TypeDecl,
    context: *const (),
) -> ErrorCode {
    accept_user_defined_type_impl(type_decl, context, &VISITOR)
}

/// Traverses the children of a user-defined type declaration.
///
/// # Safety
///
/// - `type_decl` must be a valid pointer to a `TypeDecl` representing a user-defined type.
/// - `visitor` must be a valid pointer to a `Visitor` struct.
#[cfg(not(target_arch = "wasm32"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accept_user_defined_type(
    type_decl: *const TypeDecl,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    if visitor.is_null() {
        return ErrorCode::NullPtr;
    }
    accept_user_defined_type_impl(type_decl, context, unsafe { &*visitor })
}

macro_rules! accept_impl {
    ($name:ident, $node_type:ty, $raw_node_type:ty) => {
        paste! {
            fn [<accept_ $name _impl>](
                node: *const $node_type,
                context: *const (),
                visitor: &Visitor,
            ) -> ErrorCode {
                if node.is_null() {
                    return ErrorCode::NullPtr;
                }
                let mut wrapper = VisitorWrapper::new(context, visitor);
                let raw_node: &$raw_node_type = unsafe { (*node).raw_ptr.as_ref() };
                crate::ast::visitor::[<accept_ $name>](raw_node, &mut wrapper);
                ErrorCode::Ok
            }

            #[cfg(target_arch = "wasm32")]
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn [<accept_ $name>](node: *const $node_type, context: *const ()) -> ErrorCode {
                [<accept_ $name _impl>](node, context, &VISITOR)
            }

            /// Traverses the children of the given AST node.
            ///
            /// # Safety
            ///
            /// - `node` must be a valid pointer to the corresponding AST node.
            /// - `visitor` must be a valid pointer to a `Visitor` struct.
            #[cfg(not(target_arch = "wasm32"))]
            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn [<accept_ $name>](
                node: *const $node_type,
                context: *const (),
                visitor: *const Visitor,
            ) -> ErrorCode {
                if visitor.is_null() {
                    return ErrorCode::NullPtr;
                }
                [<accept_ $name _impl>](node, context, unsafe { &*visitor })
            }
        }
    };
}

accept_impl!(program_unit, ProgramUnit, ast::ProgramUnit);
accept_impl!(service_unit, ServiceUnit, ast::ServiceUnit);
accept_impl!(ctor_func, CtorFunc, ast::CtorFunc);
accept_impl!(func_param, FuncParam, ast::FuncParam);
accept_impl!(r#type, Type, ast::Type);
accept_impl!(service_func, ServiceFunc, ast::ServiceFunc);
accept_impl!(service_event, ServiceEvent, ast::ServiceEvent);
accept_impl!(struct_def, StructDef, ast::StructDef);
accept_impl!(struct_field, StructField, ast::StructField);
accept_impl!(enum_def, EnumDef, ast::EnumDef);
accept_impl!(enum_variant, EnumVariant, ast::EnumVariant);
accept_impl!(
    program_service_item,
    ProgramServiceItem,
    ast::ProgramServiceItem
);
accept_impl!(type_decl, TypeDecl, ast::TypeDecl);
accept_impl!(type_parameter, TypeParameter, ast::TypeParameter);
accept_impl!(type_def, TypeDef, ast::TypeDef);

fn accept_idl_doc_impl(doc: *const IdlDoc, context: *const (), visitor: &Visitor) -> ErrorCode {
    if doc.is_null() {
        return ErrorCode::NullPtr;
    }
    let mut wrapper = VisitorWrapper::new(context, visitor);
    let ast_doc: &ast::IdlDoc = unsafe { (*doc).raw_ptr.as_ref() };
    crate::ast::visitor::accept_idl_doc(ast_doc, &mut wrapper);
    ErrorCode::Ok
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accept_idl_doc(doc: *const IdlDoc, context: *const ()) -> ErrorCode {
    accept_idl_doc_impl(doc, context, &VISITOR)
}

/// Traverses the children of the root `IdlDoc` node.
///
/// # Safety
///
/// - `doc` must be a valid pointer to an `IdlDoc` struct.
/// - `visitor` must be a valid pointer to a `Visitor` struct.
#[cfg(not(target_arch = "wasm32"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accept_idl_doc(
    doc: *const IdlDoc,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    if visitor.is_null() {
        return ErrorCode::NullPtr;
    }
    accept_idl_doc_impl(doc, context, unsafe { &*visitor })
}
