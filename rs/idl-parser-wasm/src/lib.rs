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
    let idl_str = match decode_idl_input(idl_utf8, idl_len) {
        Ok(s) => s,
        Err(result) => return result,
    };

    let idl_doc = match sails_idl_parser_v2::parse_idl(&idl_str) {
        Ok(doc) => doc,
        Err(err) => return create_parse_result(ErrorCode::ParseError, err.to_string()),
    };
    match idl_doc.to_json_string() {
        Ok(json) => create_parse_result(ErrorCode::Ok, json),
        Err(err) => create_parse_result(ErrorCode::ParseError, err.to_string()),
    }
}

/// # Safety
///
/// Function [`free_parse_result`] should be called after this function
#[unsafe(no_mangle)]
pub unsafe extern "C" fn compute_interface_ids_to_json(
    idl_utf8: *const u8,
    idl_len: i32,
) -> *mut ParseResult {
    let idl_str = match decode_idl_input(idl_utf8, idl_len) {
        Ok(s) => s,
        Err(result) => return result,
    };

    let ids = match sails_idl_parser_v2::compute_interface_ids(&idl_str) {
        Ok(ids) => ids,
        Err(err) => return create_parse_result(ErrorCode::ParseError, err.to_string()),
    };

    // Build a `{ "ServiceName": "0xhex", ... }` map. Service idents per the
    // current Pest grammar are ASCII-safe and never need JSON escaping, but
    // serde_json is already in this WASM blob (via sails-idl-ast's serde
    // feature used by parse_idl_to_json) so we use it to stay correct if the
    // grammar ever evolves.
    let id_strings: std::collections::BTreeMap<&str, String> = ids
        .iter()
        .map(|(name, id)| (name.as_str(), id.to_string()))
        .collect();
    match serde_json::to_string(&id_strings) {
        Ok(json) => create_parse_result(ErrorCode::Ok, json),
        Err(err) => create_parse_result(ErrorCode::ParseError, err.to_string()),
    }
}

fn create_parse_result(code: ErrorCode, str: String) -> *mut ParseResult {
    // CString rejects interior NUL bytes. Validation errors can include user
    // input that legitimately contains '\0' (e.g. NUL inside an annotation
    // value picked up by `StrToEol`), so sanitize before constructing the
    // C string instead of panicking.
    let sanitized = if str.bytes().any(|b| b == 0) {
        str.replace('\0', "\\0")
    } else {
        str
    };
    let cstring =
        std::ffi::CString::new(sanitized).expect("CString::new must succeed after NUL sanitization");
    let result = ParseResult {
        code,
        str: cstring.into_raw(),
    };
    Box::into_raw(Box::new(result))
}

/// Decode raw FFI input into a `String` without invoking undefined behavior.
///
/// Returns `Err` with a ready-to-return `*mut ParseResult` when the inputs are
/// invalid — null pointer, negative length, or non-UTF-8 bytes. Zero-length
/// input with a non-null pointer is accepted and returns an empty string so
/// the FFI matches the safe-Rust API (`compute_interface_ids("")` returns an
/// empty map).
fn decode_idl_input(idl_utf8: *const u8, idl_len: i32) -> Result<String, *mut ParseResult> {
    if idl_utf8.is_null() || idl_len < 0 {
        return Err(create_parse_result(
            ErrorCode::NullPtr,
            "null pointer or incorrect len provided".to_string(),
        ));
    }
    if idl_len == 0 {
        return Ok(String::new());
    }
    let slice = unsafe { std::slice::from_raw_parts(idl_utf8, idl_len as usize) };
    match std::str::from_utf8(slice) {
        Ok(s) => Ok(s.to_string()),
        Err(err) => Err(create_parse_result(
            ErrorCode::ParseError,
            format!("idl source is not valid UTF-8: {err}"),
        )),
    }
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
