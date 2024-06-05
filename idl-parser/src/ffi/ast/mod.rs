use crate::ast;
use std::{
    ffi::{c_char, CStr, CString},
    slice, str,
};

pub mod visitor;

#[repr(C)]
pub enum ParseResult {
    Success(*mut Program),
    Error(*mut Error),
}

#[repr(C)]
pub enum AcceptResult {
    Success,
    Error(*mut Error),
}

#[repr(C)]
pub struct Error {
    code: ErrorCode,
    details: *const c_char,
    // If true, `details` is an allocated CString.
    is_dynamic: bool,
}

#[repr(C)]
pub enum ErrorCode {
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

    let program_box = Box::new(program);
    let result = Box::new(ParseResult::Success(Box::into_raw(program_box)));

    Box::into_raw(result)
}

fn create_parse_error(
    code: ErrorCode,
    e: impl std::error::Error,
    context: &'static str,
) -> *mut ParseResult {
    let details = CString::new(format!("{}: {}", context, e)).unwrap();
    let error = Error {
        code,
        is_dynamic: true,
        details: details.into_raw(),
    };
    let result = Box::new(ParseResult::Error(Box::into_raw(Box::new(error))));
    Box::into_raw(result)
}

fn ok() -> *mut AcceptResult {
    Box::into_raw(Box::new(AcceptResult::Success))
}

fn create_accept_error(code: ErrorCode, msg: &'static CStr) -> *mut AcceptResult {
    let error = Error {
        code,
        is_dynamic: false,
        details: msg.as_ptr() as *const c_char,
    };
    let result = Box::new(AcceptResult::Error(Box::into_raw(Box::new(error))));
    Box::into_raw(result)
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

        match *result {
            ParseResult::Success(t) => {
                let program = Box::from_raw(t);
                drop(program);
            }
            ParseResult::Error(errptr) => {
                let err = Box::from_raw(errptr);

                if err.is_dynamic {
                    let err = CString::from_raw(err.details as *mut i8);
                    drop(err);
                }
            }
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn free_accept_result(result: *mut AcceptResult) {
    if result.is_null() {
        return;
    }
    unsafe {
        let result = Box::from_raw(result);

        if let AcceptResult::Error(errptr) = *result {
            let error = Box::from_raw(errptr);
            let details = CString::from_raw(error.details as *mut i8);
            drop(details);
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
