use crate::ast;
use std::{
    ffi::{c_char, CString},
    slice, str,
};

pub mod visitor;

#[repr(C)]
pub struct ParseResult {
    program: *mut Program,
    error: Error,
}

#[repr(C)]
pub struct AcceptResult {
    error: Error,
}

#[repr(C)]
pub struct Error {
    code: ErrorCode,
    details: *const c_char,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorCode {
    Ok,
    InvalidIDL,
    ParseError,
    NullPtr,
}

/// # Safety
///
/// See the safety documentation of [`slice::from_raw_parts`].
#[no_mangle]
pub unsafe extern "C" fn parse_idl(idl_ptr: *const u8, idl_len: u32) -> *mut ParseResult {
    let idl_slice = unsafe { slice::from_raw_parts(idl_ptr, idl_len as usize) };
    let idl_str = match str::from_utf8(idl_slice) {
        Ok(s) => s,
        Err(e) => return create_parse_error(ErrorCode::ParseError, e, "validate IDL string"),
    };

    let program = match ast::parse_idl(idl_str) {
        Ok(p) => p,
        Err(e) => return create_parse_error(ErrorCode::ParseError, e, "parse IDL"),
    };

    let result = ParseResult {
        error: Error {
            code: ErrorCode::Ok,
            details: std::ptr::null(),
        },
        program: Box::into_raw(Box::new(program)),
    };

    Box::into_raw(Box::new(result))
}

fn create_parse_error(
    code: ErrorCode,
    e: impl std::error::Error,
    context: &'static str,
) -> *mut ParseResult {
    let details = CString::new(format!("{}: {}", context, e)).unwrap();
    let result = ParseResult {
        error: Error {
            code,
            details: details.into_raw(),
        },
        program: std::ptr::null_mut(),
    };
    Box::into_raw(Box::new(result))
}

fn ok() -> *mut AcceptResult {
    Box::into_raw(Box::new(AcceptResult {
        error: Error {
            code: ErrorCode::Ok,
            details: std::ptr::null(),
        },
    }))
}

fn create_accept_error(code: ErrorCode) -> *mut AcceptResult {
    let result = AcceptResult {
        error: Error {
            code,
            details: std::ptr::null(),
        },
    };
    Box::into_raw(Box::new(result))
}

/// # Safety
///
/// Pointer must be obtained from [`parse_idl`].
#[no_mangle]
pub unsafe extern "C" fn free_parse_result(result: *mut ParseResult) {
    if result.is_null() {
        return;
    }
    unsafe {
        let result = Box::from_raw(result);
        if result.error.code != ErrorCode::Ok {
            let details = CString::from_raw(result.error.details as *mut i8);
            drop(details);
        }
    }
}

/// # Safety
///
/// Pointer must be obtained from `accept_*` function
#[no_mangle]
pub unsafe extern "C" fn free_accept_result(result: *mut AcceptResult) {
    if result.is_null() {
        return;
    }
    unsafe {
        let result = Box::from_raw(result);
        drop(result);
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
