use crate::ast;
use std::{slice, str};

pub mod visitor;

/// # Safety
///
/// See the safety documentation of [`slice::from_raw_parts`].
#[no_mangle]
pub unsafe extern "C" fn parse_idl(idl_ptr: *const u8, idl_len: u32) -> *mut Program {
    let idl = unsafe { slice::from_raw_parts(idl_ptr, idl_len.try_into().unwrap()) };
    let idl = str::from_utf8(idl).unwrap();
    let program = ast::parse_idl(idl).unwrap();
    let program = Box::new(program);
    Box::into_raw(program)
}

/// # Safety
///
/// TODO
#[no_mangle]
pub unsafe extern "C" fn free_program(program: *mut Program) {
    if program.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(program));
    }
}

pub type Program = ast::Program;

pub type Service = ast::Service;

#[repr(C, packed)]
pub struct Func {
    name_ptr: *const u8,
    name_len: u32,
    is_query: bool,
    raw_func: *const ast::Func,
}

#[repr(C, packed)]
pub struct FuncParam {
    name_ptr: *const u8,
    name_len: u32,
    raw_func_param: *const ast::FuncParam,
}

#[repr(C, packed)]
pub struct Type {
    name_ptr: *const u8,
    name_len: u32,
    raw_type: *const ast::Type,
}

pub type TypeDecl = ast::TypeDecl;

pub type PrimitiveType = ast::PrimitiveType;

pub type StructDef = ast::StructDef;

#[repr(C, packed)]
pub struct StructField {
    name_ptr: *const u8,
    name_len: u32,
    raw_struct_field: *const ast::StructField,
}

pub type EnumDef = ast::EnumDef;

#[repr(C, packed)]
pub struct EnumVariant {
    name_ptr: *const u8,
    name_len: u32,
    raw_enum_variant: *const ast::EnumVariant,
}
