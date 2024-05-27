use crate::ast;
use std::{
    error::Error,
    ffi::{c_char, CString},
    slice, str,
};

pub mod visitor;

#[repr(C)]
pub enum ParseResult {
    Success(*mut Program),
    Error(*const c_char),
}

/// # Safety
///
/// See the safety documentation of [`slice::from_raw_parts`].
#[no_mangle]
pub unsafe extern "C" fn parse_idl(idl_ptr: *const u8, idl_len: u32) -> *mut ParseResult {
    let idl_slice = unsafe { slice::from_raw_parts(idl_ptr, idl_len as usize) };
    let idl_str = match str::from_utf8(idl_slice) {
        Ok(s) => s,
        Err(e) => return create_error(e, "validate IDL string"),
    };

    let program = match ast::parse_idl(idl_str) {
        Ok(p) => p,
        Err(e) => return create_error(e, "parse IDL"),
    };

    let program_box = Box::new(program);
    let result = Box::new(ParseResult::Success(Box::into_raw(program_box)));
    Box::into_raw(result)
}

fn create_error(e: impl Error, context: &str) -> *mut ParseResult {
    let err_str = CString::new(format!("{}: {}", context, e)).unwrap();
    let result = Box::new(ParseResult::Error(err_str.into_raw()));
    Box::into_raw(result)
}

/// # Safety
///
/// Pointer must be obtained from [`parse_idl`].
#[no_mangle]
pub unsafe extern "C" fn free_result(result: *mut ParseResult) {
    if result.is_null() {
        return;
    }
    unsafe {
        let result = Box::from_raw(result);

        match *result {
            ParseResult::Success(program) => {
                let program = Box::from_raw(program);
                drop(program);
            }
            ParseResult::Error(err) => {
                let err = CString::from_raw(err as *mut i8);
                drop(err);
            }
        }
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
    name_ptr: *const u8,
    name_len: u32,
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
