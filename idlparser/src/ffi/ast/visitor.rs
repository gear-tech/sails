use super::*;
use crate::{ast as raw_ast, ast::visitor as raw_visitor, ast::visitor::Visitor as RawVisitor};
use std::ptr;

#[repr(C, packed)]
pub struct Visitor {
    visit_service: extern "C" fn(*mut Visitor, *const Service),
    visit_type: extern "C" fn(*mut Visitor, *const Type),
    visit_optional_type_decl: extern "C" fn(*mut Visitor, *const TypeDecl),
    visit_vector_type_decl: extern "C" fn(*mut Visitor, *const TypeDecl),
    visit_result_type_decl: extern "C" fn(*mut Visitor, *const TypeDecl, *const TypeDecl),
    visit_primitive_type_id: extern "C" fn(*mut Visitor, *const PrimitiveType),
    visit_user_defined_type_id: extern "C" fn(*mut Visitor, *const u8, u32),
    visit_func: extern "C" fn(*mut Visitor, *const Func),
    visit_func_param: extern "C" fn(*mut Visitor, *const FuncParam),
    visit_func_output: extern "C" fn(*mut Visitor, *const TypeDecl),
    visit_struct_def: extern "C" fn(*mut Visitor, *const StructDef),
    visit_struct_field: extern "C" fn(*mut Visitor, *const StructField),
    visit_enum_def: extern "C" fn(*mut Visitor, *const EnumDef),
    visit_enum_variant: extern "C" fn(*mut Visitor, *const EnumVariant),
}

/// # Safety
///
/// See documentation for [`const_ptr::as_ref`] and [`mut_ptr::as_mut`]
#[no_mangle]
pub unsafe extern "C" fn accept_program(program: *const Program, visitor: *mut Visitor) {
    let program = unsafe { program.as_ref() }.unwrap();
    let visitor = unsafe { visitor.as_mut() }.unwrap();
    let mut visitor = VisitorWrapper(visitor);
    raw_visitor::accept_program(program, &mut visitor);
}

/// # Safety
///
/// See documentation for [`const_ptr::as_ref`] and [`mut_ptr::as_mut`]
#[no_mangle]
pub unsafe extern "C" fn accept_service(service: *const Service, visitor: *mut Visitor) {
    let service = unsafe { service.as_ref() }.unwrap();
    let visitor = unsafe { visitor.as_mut() }.unwrap();
    let mut visitor = VisitorWrapper(visitor);
    raw_visitor::accept_service(service, &mut visitor);
}

/// # Safety
///
/// See documentation for [`const_ptr::as_ref`] and [`mut_ptr::as_mut`]
#[no_mangle]
pub unsafe extern "C" fn accept_func(func: *const Func, visitor: *mut Visitor) {
    let func = unsafe { func.as_ref() }.unwrap();
    let visitor = unsafe { visitor.as_mut() }.unwrap();
    let mut visitor = VisitorWrapper(visitor);
    raw_visitor::accept_func(unsafe { func.raw_func.as_ref() }.unwrap(), &mut visitor);
}

/// # Safety
///
/// See documentation for [`const_ptr::as_ref`] and [`mut_ptr::as_mut`]
#[no_mangle]
pub unsafe extern "C" fn accept_func_param(func_param: *const FuncParam, visitor: *mut Visitor) {
    let func_param = unsafe { func_param.as_ref() }.unwrap();
    let visitor = unsafe { visitor.as_mut() }.unwrap();
    let mut visitor = VisitorWrapper(visitor);
    raw_visitor::accept_func_param(
        unsafe { func_param.raw_func_param.as_ref() }.unwrap(),
        &mut visitor,
    );
}

/// # Safety
///
/// See documentation for [`const_ptr::as_ref`] and [`mut_ptr::as_mut`]
#[no_mangle]
pub unsafe extern "C" fn accept_type(r#type: *const Type, visitor: *mut Visitor) {
    let r#type = unsafe { r#type.as_ref() }.unwrap();
    let visitor = unsafe { visitor.as_mut() }.unwrap();
    let mut visitor = VisitorWrapper(visitor);
    raw_visitor::accept_type(unsafe { r#type.raw_type.as_ref() }.unwrap(), &mut visitor);
}

/// # Safety
///
/// See documentation for [`const_ptr::as_ref`] and [`mut_ptr::as_mut`]
#[no_mangle]
pub unsafe extern "C" fn accept_type_decl(type_decl: *const TypeDecl, visitor: *mut Visitor) {
    let type_decl = unsafe { type_decl.as_ref() }.unwrap();
    let visitor = unsafe { visitor.as_mut() }.unwrap();
    let mut visitor = VisitorWrapper(visitor);
    raw_visitor::accept_type_decl(type_decl, &mut visitor);
}

/// # Safety
///
/// See documentation for [`const_ptr::as_ref`] and [`mut_ptr::as_mut`]
#[no_mangle]
pub unsafe extern "C" fn accept_struct_def(struct_def: *const StructDef, visitor: *mut Visitor) {
    let struct_def = unsafe { struct_def.as_ref() }.unwrap();
    let visitor = unsafe { visitor.as_mut() }.unwrap();
    let mut visitor = VisitorWrapper(visitor);
    raw_visitor::accept_struct_def(struct_def, &mut visitor);
}

/// # Safety
///
/// See documentation for [`const_ptr::as_ref`] and [`mut_ptr::as_mut`]
#[no_mangle]
pub unsafe extern "C" fn accept_struct_field(
    struct_field: *const StructField,
    visitor: *mut Visitor,
) {
    let struct_field = unsafe { struct_field.as_ref() }.unwrap();
    let visitor = unsafe { visitor.as_mut() }.unwrap();
    let mut visitor = VisitorWrapper(visitor);
    raw_visitor::accept_struct_field(
        unsafe { struct_field.raw_struct_field.as_ref() }.unwrap(),
        &mut visitor,
    );
}

/// # Safety
///
/// See documentation for [`const_ptr::as_ref`] and [`mut_ptr::as_mut`]
#[no_mangle]
pub unsafe extern "C" fn accept_enum_def(enum_def: *const EnumDef, visitor: *mut Visitor) {
    let enum_def = unsafe { enum_def.as_ref() }.unwrap();
    let visitor = unsafe { visitor.as_mut() }.unwrap();
    let mut visitor = VisitorWrapper(visitor);
    raw_visitor::accept_enum_def(enum_def, &mut visitor);
}

/// # Safety
///
/// See documentation for [`const_ptr::as_ref`] and [`mut_ptr::as_mut`]
#[no_mangle]
pub unsafe extern "C" fn accept_enum_variant(
    enum_variant: *const EnumVariant,
    visitor: *mut Visitor,
) {
    let enum_variant = unsafe { enum_variant.as_ref() }.unwrap();
    let visitor = unsafe { visitor.as_mut() }.unwrap();
    let mut visitor = VisitorWrapper(visitor);
    raw_visitor::accept_enum_variant(
        unsafe { enum_variant.raw_enum_variant.as_ref() }.unwrap(),
        &mut visitor,
    );
}

macro_rules! fn_ptr_addr {
    ($fn_ptr: expr) => {{
        let fn_ptr_addr = $fn_ptr as *const ();
        fn_ptr_addr
    }};
}

struct VisitorWrapper<'a>(&'a mut Visitor);

impl<'a, 'ast> RawVisitor<'ast> for VisitorWrapper<'a> {
    fn visit_service(&mut self, service: &'ast raw_ast::Service) {
        if fn_ptr_addr!(self.0.visit_service).is_null() {
            return raw_visitor::accept_service(service, self);
        }
        (self.0.visit_service)(self.0, service);
    }

    fn visit_type(&mut self, r#type: &'ast raw_ast::Type) {
        if fn_ptr_addr!(self.0.visit_type).is_null() {
            return raw_visitor::accept_type(r#type, self);
        }
        let name_bytes = r#type.name().as_bytes();
        let r#type = Type {
            name_ptr: name_bytes.as_ptr(),
            name_len: name_bytes.len() as u32,
            raw_type: r#type,
        };
        (self.0.visit_type)(self.0, &r#type);
    }

    fn visit_optional_type_decl(&mut self, optional_type_decl: &'ast raw_ast::TypeDecl) {
        if fn_ptr_addr!(self.0.visit_optional_type_decl).is_null() {
            return raw_visitor::accept_type_decl(optional_type_decl, self);
        }
        (self.0.visit_optional_type_decl)(self.0, optional_type_decl);
    }

    fn visit_vector_type_decl(&mut self, vector_type_decl: &'ast raw_ast::TypeDecl) {
        if fn_ptr_addr!(self.0.visit_vector_type_decl).is_null() {
            return raw_visitor::accept_type_decl(vector_type_decl, self);
        }
        (self.0.visit_vector_type_decl)(self.0, vector_type_decl);
    }

    fn visit_result_type_decl(
        &mut self,
        ok_type_decl: &'ast raw_ast::TypeDecl,
        err_type_decl: &'ast raw_ast::TypeDecl,
    ) {
        if fn_ptr_addr!(self.0.visit_result_type_decl).is_null() {
            return raw_visitor::accept_type_decl(ok_type_decl, self);
        }
        (self.0.visit_result_type_decl)(self.0, ok_type_decl, err_type_decl);
    }

    fn visit_primitive_type_id(&mut self, primitive_type_id: &'ast raw_ast::PrimitiveType) {
        if fn_ptr_addr!(self.0.visit_primitive_type_id).is_null() {
            return;
        }
        (self.0.visit_primitive_type_id)(self.0, primitive_type_id);
    }

    fn visit_user_defined_type_id(&mut self, user_defined_type_id: &'ast str) {
        if fn_ptr_addr!(self.0.visit_user_defined_type_id).is_null() {
            return;
        }
        let user_defined_type_id = user_defined_type_id.as_bytes();
        (self.0.visit_user_defined_type_id)(
            self.0,
            user_defined_type_id.as_ptr(),
            user_defined_type_id.len() as u32,
        );
    }

    fn visit_func(&mut self, func: &'ast raw_ast::Func) {
        let func_name_bytes = func.name().as_bytes();
        let func = Func {
            name_ptr: func_name_bytes.as_ptr(),
            name_len: func_name_bytes.len() as u32,
            is_query: func.is_query(),
            raw_func: func,
        };
        (self.0.visit_func)(self.0, &func);
    }

    fn visit_func_param(&mut self, func_param: &'ast raw_ast::FuncParam) {
        if fn_ptr_addr!(self.0.visit_func_param).is_null() {
            return raw_visitor::accept_func_param(func_param, self);
        }
        let func_param_name_bytes = func_param.name().as_bytes();
        let func_param = FuncParam {
            name_ptr: func_param_name_bytes.as_ptr(),
            name_len: func_param_name_bytes.len() as u32,
            raw_func_param: func_param,
        };
        (self.0.visit_func_param)(self.0, &func_param);
    }

    fn visit_func_output(&mut self, func_output: &'ast raw_ast::TypeDecl) {
        if fn_ptr_addr!(self.0.visit_func_output).is_null() {
            return raw_visitor::accept_type_decl(func_output, self);
        }
        (self.0.visit_func_output)(self.0, func_output);
    }

    fn visit_struct_def(&mut self, struct_def: &'ast raw_ast::StructDef) {
        if fn_ptr_addr!(self.0.visit_struct_def).is_null() {
            return raw_visitor::accept_struct_def(struct_def, self);
        }
        (self.0.visit_struct_def)(self.0, struct_def);
    }

    fn visit_struct_field(&mut self, struct_field: &'ast raw_ast::StructField) {
        if fn_ptr_addr!(self.0.visit_struct_field).is_null() {
            return raw_visitor::accept_struct_field(struct_field, self);
        }
        let struct_field_name_bytes = struct_field.name().map(|n| n.as_bytes());
        let struct_field = StructField {
            name_ptr: struct_field_name_bytes.map_or(ptr::null(), |n| n.as_ptr()),
            name_len: struct_field_name_bytes.map_or(0, |n| n.len() as u32),
            raw_struct_field: struct_field,
        };
        (self.0.visit_struct_field)(self.0, &struct_field);
    }

    fn visit_enum_def(&mut self, enum_def: &'ast raw_ast::EnumDef) {
        if fn_ptr_addr!(self.0.visit_enum_def).is_null() {
            return raw_visitor::accept_enum_def(enum_def, self);
        }
        (self.0.visit_enum_def)(self.0, enum_def);
    }

    fn visit_enum_variant(&mut self, enum_variant: &'ast raw_ast::EnumVariant) {
        if fn_ptr_addr!(self.0.visit_enum_variant).is_null() {
            return raw_visitor::accept_enum_variant(enum_variant, self);
        }
        let enum_variant_name_bytes = enum_variant.name().as_bytes();
        let enum_variant = EnumVariant {
            name_ptr: enum_variant_name_bytes.as_ptr(),
            name_len: enum_variant_name_bytes.len() as u32,
            raw_enum_variant: enum_variant,
        };
        (self.0.visit_enum_variant)(self.0, &enum_variant);
    }
}
