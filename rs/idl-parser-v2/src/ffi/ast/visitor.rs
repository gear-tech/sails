use super::{
    Allocations, Annotation, CtorFunc, EnumDef, EnumVariant, ErrorCode, FuncParam, IdlDoc,
    ProgramUnit, ServiceEvent, ServiceExpo, ServiceFunc, ServiceUnit, StructDef, StructField, Type,
    TypeDecl, TypeDef, TypeParameter,
};
use crate::ast;
use crate::visitor::Visitor as RawVisitor;
use alloc::{string::String, vec::Vec};
use core::ptr;
use paste::paste;

#[repr(C)]
pub struct Visitor {
    pub visit_globals:
        Option<unsafe extern "C" fn(context: *const (), globals: *const Annotation, len: u32)>,
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
    pub visit_primitive_type:
        Option<unsafe extern "C" fn(context: *const (), primitive: ast::PrimitiveType)>,
    pub visit_named_type_decl: Option<
        unsafe extern "C" fn(
            context: *const (),
            path_ptr: *const u8,
            path_len: u32,
            generics_ptr: *const TypeDecl,
            generics_len: u32,
        ),
    >,
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
    pub visit_service_expo:
        Option<unsafe extern "C" fn(context: *const (), service_item: *const ServiceExpo)>,
    pub visit_type_parameter:
        Option<unsafe extern "C" fn(context: *const (), type_param: *const TypeParameter)>,
    pub visit_type_def: Option<unsafe extern "C" fn(context: *const (), type_def: *const TypeDef)>,
}

#[cfg(target_arch = "wasm32")]
unsafe extern "C" {
    fn visit_globals(context: *const (), globals: *const Annotation, len: u32);
    fn visit_program_unit(context: *const (), program: *const ProgramUnit);
    fn visit_service_unit(context: *const (), service: *const ServiceUnit);
    fn visit_ctor_func(context: *const (), ctor: *const CtorFunc);
    fn visit_func_param(context: *const (), param: *const FuncParam);
    fn visit_type(context: *const (), ty: *const Type);
    fn visit_slice_type_decl(context: *const (), item_ty: *const TypeDecl);
    fn visit_array_type_decl(context: *const (), item_ty: *const TypeDecl, len: u32);
    fn visit_tuple_type_decl(context: *const (), items_ptr: *const TypeDecl, items_len: u32);
    fn visit_primitive_type(context: *const (), primitive: ast::PrimitiveType);
    fn visit_named_type_decl(
        context: *const (),
        path_ptr: *const u8,
        path_len: u32,
        generics_ptr: *const TypeDecl,
        generics_len: u32,
    );
    fn visit_service_func(context: *const (), func: *const ServiceFunc);
    fn visit_service_event(context: *const (), event: *const ServiceEvent);
    fn visit_struct_def(context: *const (), def: *const StructDef);
    fn visit_struct_field(context: *const (), field: *const StructField);
    fn visit_enum_def(context: *const (), def: *const EnumDef);
    fn visit_enum_variant(context: *const (), variant: *const EnumVariant);
    fn visit_service_expo(context: *const (), service_item: *const ServiceExpo);
    fn visit_type_parameter(context: *const (), type_param: *const TypeParameter);
    fn visit_type_def(context: *const (), type_def: *const TypeDef);
}

#[cfg(target_arch = "wasm32")]
static VISITOR: Visitor = Visitor {
    visit_globals: Some(visit_globals),
    visit_program_unit: Some(visit_program_unit),
    visit_service_unit: Some(visit_service_unit),
    visit_ctor_func: Some(visit_ctor_func),
    visit_func_param: Some(visit_func_param),
    visit_type: Some(visit_type),
    visit_slice_type_decl: Some(visit_slice_type_decl),
    visit_array_type_decl: Some(visit_array_type_decl),
    visit_tuple_type_decl: Some(visit_tuple_type_decl),
    visit_primitive_type: Some(visit_primitive_type),
    visit_named_type_decl: Some(visit_named_type_decl),
    visit_service_func: Some(visit_service_func),
    visit_service_event: Some(visit_service_event),
    visit_struct_def: Some(visit_struct_def),
    visit_struct_field: Some(visit_struct_field),
    visit_enum_def: Some(visit_enum_def),
    visit_enum_variant: Some(visit_enum_variant),
    visit_service_expo: Some(visit_service_expo),
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
    fn visit_globals(&mut self, globals: &'ast [(String, Option<String>)]) {
        if let Some(visit) = self.visitor.visit_globals {
            let (ptr, len) = super::allocate_annotation_vec(globals, &mut self.allocations);
            unsafe { visit(self.context, ptr, len) };
        }
    }

    fn visit_program_unit(&mut self, program: &'ast ast::ProgramUnit) {
        if let Some(visit) = self.visitor.visit_program_unit {
            let ffi_program = ProgramUnit::from_ast(program, &mut self.allocations);
            unsafe { visit(self.context, &ffi_program) };
            return;
        }
        crate::visitor::accept_program_unit(program, self);
    }

    fn visit_service_unit(&mut self, service: &'ast ast::ServiceUnit) {
        if let Some(visit) = self.visitor.visit_service_unit {
            let ffi_service = ServiceUnit::from_ast(service, &mut self.allocations);
            unsafe { visit(self.context, &ffi_service) };
            return;
        }
        crate::visitor::accept_service_unit(service, self);
    }

    fn visit_ctor_func(&mut self, ctor: &'ast ast::CtorFunc) {
        if let Some(visit) = self.visitor.visit_ctor_func {
            let ffi_ctor = CtorFunc::from_ast(ctor, &mut self.allocations);
            unsafe { visit(self.context, &ffi_ctor) };
            return;
        }
        crate::visitor::accept_ctor_func(ctor, self);
    }

    fn visit_func_param(&mut self, param: &'ast ast::FuncParam) {
        if let Some(visit) = self.visitor.visit_func_param {
            let ffi_param = FuncParam::from_ast(param, &mut self.allocations);
            unsafe { visit(self.context, &ffi_param) };
            return;
        }
        crate::visitor::accept_func_param(param, self);
    }

    fn visit_type(&mut self, ty: &'ast ast::Type) {
        if let Some(visit) = self.visitor.visit_type {
            let ffi_ty = Type::from_ast(ty, &mut self.allocations);
            unsafe { visit(self.context, &ffi_ty) };
            return;
        }
        crate::visitor::accept_type(ty, self);
    }

    fn visit_slice_type_decl(&mut self, item_type_decl: &'ast ast::TypeDecl) {
        if let Some(visit) = self.visitor.visit_slice_type_decl {
            let ffi_item_ty = TypeDecl::from_ast(item_type_decl, &mut self.allocations);
            unsafe { visit(self.context, &ffi_item_ty) };
            return;
        }
        crate::visitor::accept_type_decl(item_type_decl, self);
    }

    fn visit_array_type_decl(&mut self, item_type_decl: &'ast ast::TypeDecl, len: u32) {
        if let Some(visit) = self.visitor.visit_array_type_decl {
            let ffi_item_ty = TypeDecl::from_ast(item_type_decl, &mut self.allocations);
            unsafe { visit(self.context, &ffi_item_ty, len) };
            return;
        }
        crate::visitor::accept_type_decl(item_type_decl, self);
    }

    fn visit_tuple_type_decl(&mut self, items: &'ast [ast::TypeDecl]) {
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
        for item in items {
            crate::visitor::accept_type_decl(item, self);
        }
    }

    fn visit_primitive_type(&mut self, primitive_type: ast::PrimitiveType) {
        if let Some(visit) = self.visitor.visit_primitive_type {
            unsafe { visit(self.context, primitive_type) };
        }
    }

    fn visit_named_type_decl(&mut self, path: &'ast str, generics: &'ast [ast::TypeDecl]) {
        let path_bytes = path.as_bytes().to_vec();
        let boxed_path = path_bytes.into_boxed_slice();
        let path_ptr = boxed_path.as_ptr();
        let path_len = boxed_path.len() as u32;
        self.allocations.strings.push(boxed_path);

        if let Some(visit) = self.visitor.visit_named_type_decl {
            let ffi_generics: Vec<TypeDecl> = generics
                .iter()
                .map(|g| TypeDecl::from_ast(g, &mut self.allocations))
                .collect();

            let (generics_ptr, generics_len) = if !ffi_generics.is_empty() {
                let boxed_slice = ffi_generics.into_boxed_slice();
                let ptr = boxed_slice.as_ptr();
                let len = boxed_slice.len() as u32;
                self.allocations.type_decls.push(boxed_slice);
                (ptr, len)
            } else {
                (ptr::null(), 0)
            };

            unsafe { visit(self.context, path_ptr, path_len, generics_ptr, generics_len) };
            return;
        }
        for generic in generics {
            crate::visitor::accept_type_decl(generic, self);
        }
    }

    fn visit_service_func(&mut self, func: &'ast ast::ServiceFunc) {
        if let Some(visit) = self.visitor.visit_service_func {
            let ffi_func = ServiceFunc::from_ast(func, &mut self.allocations);
            unsafe { visit(self.context, &ffi_func) };
            return;
        }
        crate::visitor::accept_service_func(func, self);
    }

    fn visit_service_event(&mut self, event: &'ast ast::ServiceEvent) {
        if let Some(visit) = self.visitor.visit_service_event {
            let ffi_event = ServiceEvent::from_ast(event, &mut self.allocations);
            unsafe { visit(self.context, &ffi_event) };
            return;
        }
        crate::visitor::accept_service_event(event, self);
    }

    fn visit_struct_def(&mut self, def: &'ast ast::StructDef) {
        if let Some(visit) = self.visitor.visit_struct_def {
            let ffi_def = StructDef::from_ast(def, &mut self.allocations);
            unsafe { visit(self.context, &ffi_def) };
            return;
        }
        crate::visitor::accept_struct_def(def, self);
    }

    fn visit_struct_field(&mut self, field: &'ast ast::StructField) {
        if let Some(visit) = self.visitor.visit_struct_field {
            let ffi_field = StructField::from_ast(field, &mut self.allocations);
            unsafe { visit(self.context, &ffi_field) };
            return;
        }
        crate::visitor::accept_struct_field(field, self);
    }

    fn visit_enum_def(&mut self, def: &'ast ast::EnumDef) {
        if let Some(visit) = self.visitor.visit_enum_def {
            let ffi_def = EnumDef::from_ast(def, &mut self.allocations);
            unsafe { visit(self.context, &ffi_def) };
            return;
        }
        crate::visitor::accept_enum_def(def, self);
    }

    fn visit_enum_variant(&mut self, variant: &'ast ast::EnumVariant) {
        if let Some(visit) = self.visitor.visit_enum_variant {
            let ffi_variant = EnumVariant::from_ast(variant, &mut self.allocations);
            unsafe { visit(self.context, &ffi_variant) };
            return;
        }
        crate::visitor::accept_enum_variant(variant, self);
    }

    fn visit_service_expo(&mut self, service_item: &'ast ast::ServiceExpo) {
        if let Some(visit) = self.visitor.visit_service_expo {
            let ffi_item = ServiceExpo::from_ast(service_item, &mut self.allocations);
            unsafe { visit(self.context, &ffi_item) };
            return;
        }
        crate::visitor::accept_service_expo(service_item, self);
    }

    fn visit_type_parameter(&mut self, type_param: &'ast ast::TypeParameter) {
        if let Some(visit) = self.visitor.visit_type_parameter {
            let ffi_param = TypeParameter::from_ast(type_param, &mut self.allocations);
            unsafe { visit(self.context, &ffi_param) };
            return;
        }
        crate::visitor::accept_type_parameter(type_param, self);
    }

    fn visit_type_def(&mut self, type_def: &'ast ast::TypeDef) {
        if let Some(visit) = self.visitor.visit_type_def {
            let ffi_type_def = TypeDef::from_ast(type_def, &mut self.allocations);
            unsafe { visit(self.context, &ffi_type_def) };
            return;
        }
        crate::visitor::accept_type_def(type_def, self);
    }
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
                crate::visitor::[<accept_ $name>](raw_node, &mut wrapper);
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
accept_impl!(service_expo, ServiceExpo, ast::ServiceExpo);
accept_impl!(type_parameter, TypeParameter, ast::TypeParameter);
accept_impl!(type_def, TypeDef, ast::TypeDef);

fn accept_type_decl_impl(
    node: *const TypeDecl,
    context: *const (),
    visitor: &Visitor,
) -> ErrorCode {
    if node.is_null() {
        return ErrorCode::NullPtr;
    }
    let mut wrapper = VisitorWrapper::new(context, visitor);
    let raw_node: &ast::TypeDecl = unsafe { (*node).raw_ptr.as_ref() };

    match raw_node {
        ast::TypeDecl::Slice { item } => {
            wrapper.visit_slice_type_decl(item);
        }
        ast::TypeDecl::Array { item, len } => {
            wrapper.visit_array_type_decl(item, *len);
        }
        ast::TypeDecl::Tuple { types } => {
            wrapper.visit_tuple_type_decl(types);
        }
        ast::TypeDecl::Primitive(primitive_type) => wrapper.visit_primitive_type(*primitive_type),
        ast::TypeDecl::Named { name, generics } => {
            wrapper.visit_named_type_decl(name, generics);
        }
    }
    ErrorCode::Ok
}

///
/// Traverses the children of a `TypeDecl` node.
///
/// # Safety
///
/// - `node` must be a valid pointer to a `TypeDecl` struct.
/// - `visitor` must be a valid pointer to a `Visitor` struct.
#[cfg(not(target_arch = "wasm32"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accept_type_decl(
    node: *const TypeDecl,
    context: *const (),
    visitor: *const Visitor,
) -> ErrorCode {
    if visitor.is_null() {
        return ErrorCode::NullPtr;
    }
    accept_type_decl_impl(node, context, unsafe { &*visitor })
}

#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn accept_type_decl(node: *const TypeDecl, context: *const ()) -> ErrorCode {
    accept_type_decl_impl(node, context, &VISITOR)
}

fn accept_idl_doc_impl(doc: *const IdlDoc, context: *const (), visitor: &Visitor) -> ErrorCode {
    if doc.is_null() {
        return ErrorCode::NullPtr;
    }
    let mut wrapper = VisitorWrapper::new(context, visitor);
    let ast_doc: &ast::IdlDoc = unsafe { (*doc).raw_ptr.as_ref() };
    crate::visitor::accept_idl_doc(ast_doc, &mut wrapper);
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
