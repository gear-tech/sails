use crate::{Error as ContractGenerationError, SolidityFile};
use alloc::{boxed::Box, ffi::CString, string::String, vec::Vec};
use core::{
    ffi::{CStr, c_char},
    ptr,
};

#[repr(C)]
pub struct FFIVec {
    pub ptr: *const u8,
    pub len: u32,
}

#[repr(C)]
pub struct GenerateError {
    code: ErrorCode,
    details: *const c_char,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ErrorCode {
    Ok,
    ContractName,
    Idl,
    Conversion,
    Askama,
    NullPtr,
}

#[repr(C)]
pub struct GenerateResult {
    content: FFIVec,
    error: GenerateError,
}

fn create_generate_error(code: ErrorCode, details_str: impl Into<String>) -> *mut GenerateResult {
    let details = CString::new(details_str.into()).unwrap();
    let result = GenerateResult {
        error: GenerateError {
            code,
            details: details.into_raw(),
        },
        content: FFIVec {
            ptr: ptr::null(),
            len: 0,
        },
    };
    Box::into_raw(Box::new(result))
}

fn create_generate_result(content: Vec<u8>) -> *mut GenerateResult {
    let content = content.into_boxed_slice();
    let ffi_vec = FFIVec {
        ptr: content.as_ptr(),
        len: content.len() as u32,
    };

    let result = GenerateResult {
        content: ffi_vec,
        error: GenerateError {
            code: ErrorCode::Ok,
            details: ptr::null(),
        },
    };

    let _ = Box::into_raw(content);
    Box::into_raw(Box::new(result))
}

fn error_code(error: &ContractGenerationError) -> ErrorCode {
    match error {
        ContractGenerationError::Idl(_) => ErrorCode::Idl,
        ContractGenerationError::Conversion(_) => ErrorCode::Conversion,
        ContractGenerationError::Askama(_) => ErrorCode::Askama,
    }
}

/// Generates a Solidity contract from a contract name and IDL source string.
///
/// On failure, this function returns a `GenerateResult` with error details.
///
/// # Safety
/// - `contract_name_ptr` must be a valid, null-terminated UTF-8 string.
/// - `idl_content_ptr` must be a valid, null-terminated UTF-8 string.
/// - The returned pointer must be freed using `free_generate_result` to avoid memory leaks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn generate_solidity_contract(
    contract_name_ptr: *const c_char,
    idl_content_ptr: *const c_char,
    solidity_file: SolidityFile,
) -> *mut GenerateResult {
    if contract_name_ptr.is_null() {
        return create_generate_error(
            ErrorCode::NullPtr,
            "Null pointer passed as contract name to generate_solidity_contract",
        );
    }

    if idl_content_ptr.is_null() {
        return create_generate_error(
            ErrorCode::NullPtr,
            "Null pointer passed as IDL content to generate_solidity_contract",
        );
    }

    let contract_name = unsafe {
        match CStr::from_ptr(contract_name_ptr).to_str() {
            Ok(s) => s,
            Err(e) => {
                return create_generate_error(
                    ErrorCode::ContractName,
                    format!("Invalid UTF-8 in contract name: {e}"),
                );
            }
        }
    };
    let idl_content = unsafe {
        match CStr::from_ptr(idl_content_ptr).to_str() {
            Ok(s) => s,
            Err(e) => {
                return create_generate_error(
                    ErrorCode::Idl,
                    format!("Invalid UTF-8 in IDL content: {e}"),
                );
            }
        }
    };

    match crate::generate_solidity_contract(contract_name, idl_content, solidity_file) {
        Ok(content) => create_generate_result(content),
        Err(error) => create_generate_error(error_code(&error), format!("{error}")),
    }
}

/// Frees the memory for a `GenerateResult` allocated by `generate_solidity_contract`.
///
/// # Safety
/// - `result_ptr` must be a pointer returned by `generate_solidity_contract`.
/// - Passing a `NULL` pointer is safe (it will be a no-op).
/// - Do not use the pointer after it has been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn free_generate_result(result_ptr: *mut GenerateResult) {
    if result_ptr.is_null() {
        return;
    }

    let result = unsafe { Box::from_raw(result_ptr) };

    if !result.content.ptr.is_null() {
        let content = ptr::slice_from_raw_parts_mut(
            result.content.ptr as *mut u8,
            result.content.len as usize,
        );
        let _ = unsafe { Box::from_raw(content) };
    }

    if result.error.code != ErrorCode::Ok && !result.error.details.is_null() {
        let details = unsafe { CString::from_raw(result.error.details as *mut _) };
        drop(details);
    }
}
