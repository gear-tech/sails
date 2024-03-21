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

#[repr(C)]
pub struct Ctor {
    raw_ptr: Ptr,
}

#[repr(C)]
pub struct CtorFunc {
    raw_ptr: Ptr,
    name_ptr: *const u8,
    name_len: u32,
}

#[repr(C)]
pub struct Service {
    raw_ptr: Ptr,
}

#[repr(C)]
pub struct ServiceFunc {
    raw_ptr: Ptr,
    name_ptr: *const u8,
    name_len: u32,
    is_query: bool,
}

pub type ServiceEvent = EnumVariant;

#[repr(C)]
pub struct FuncParam {
    raw_ptr: Ptr,
    name_ptr: *const u8,
    name_len: u32,
}

#[repr(C)]
pub struct Type {
    raw_ptr: Ptr,
    name_ptr: *const u8,
    name_len: u32,
}

#[repr(C)]
pub struct TypeDecl {
    raw_ptr: Ptr,
}

pub type PrimitiveType = ast::PrimitiveType;

#[repr(C)]
pub struct StructDef {
    raw_ptr: Ptr,
}

#[repr(C)]
pub struct StructField {
    raw_ptr: Ptr,
    name_ptr: *const u8,
    name_len: u32,
}

#[repr(C)]
pub struct EnumDef {
    raw_ptr: Ptr,
}

#[repr(C)]
pub struct EnumVariant {
    raw_ptr: Ptr,
    name_ptr: *const u8,
    name_len: u32,
}

#[repr(transparent)]
pub struct Ptr(*const ());

impl<T> From<&T> for Ptr {
    fn from(t: &T) -> Self {
        Self(t as *const T as *const ())
    }
}

impl<T> AsRef<T> for Ptr {
    fn as_ref(&self) -> &T {
        unsafe { (self.0 as *const T).as_ref() }.unwrap()
    }
}
