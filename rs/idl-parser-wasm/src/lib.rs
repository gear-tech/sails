#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    Ok,
    ParseError,
    NullPtr,
}
#[repr(C)]
pub struct ParseResult {
    code: ErrorCode,
    str: *const std::ffi::c_char,
}

/// # Safety
///
/// Function [`free_parse_result`] should be called after this function
#[unsafe(no_mangle)]
pub unsafe extern "C" fn parse_idl_to_json(idl_utf8: *const u8, idl_len: i32) -> *mut ParseResult {
    if idl_utf8.is_null() || idl_len <= 0 {
        return create_parse_result(
            ErrorCode::NullPtr,
            "null pointer or incorrect len provided".to_string(),
        );
    }
    let slice = unsafe { std::slice::from_raw_parts(idl_utf8, idl_len as usize) };
    let idl_str = unsafe { String::from_utf8_unchecked(slice.to_vec()) };

    let idl_doc = match sails_idl_parser_v2::parse_idl(&idl_str) {
        Ok(doc) => doc,
        Err(err) => return create_parse_result(ErrorCode::ParseError, err.to_string()),
    };
    match idl_doc.to_json_string() {
        Ok(json) => create_parse_result(ErrorCode::Ok, json),
        Err(err) => create_parse_result(ErrorCode::ParseError, err.to_string()),
    }
}

fn create_parse_result(code: ErrorCode, str: String) -> *mut ParseResult {
    let str = std::ffi::CString::new(str)
        .expect("failed to create cstring")
        .into_raw();
    let result = ParseResult { code, str };
    Box::into_raw(Box::new(result))
}

/// # Safety
///
/// This function should not be called before the [`parse_idl_to_json`]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn free_parse_result(res: *mut ParseResult) {
    if res.is_null() {
        return;
    }
    unsafe {
        let res = Box::from_raw(res);
        // drop
        _ = std::ffi::CString::from_raw(res.str as _);
    }
}
