use crate::ast;
use alloc::{boxed::Box, ffi::CString, format, string::String, vec::Vec};
use core::ffi::{CStr, c_char};
use core::ptr;

pub mod visitor;

// FFI-safe representation of a string.
#[repr(C)]
pub struct FFIString {
    pub ptr: *const u8,
    pub len: u32,
}

// FFI-safe representation of an annotation (name and optional value).
#[repr(C)]
pub struct Annotation {
    pub name_ptr: *const u8,
    pub name_len: u32,
    pub value_ptr: *const u8,
    pub value_len: u32,
    pub has_value: bool,
}

// Helper to manage FFI allocations.
// We keep owned Boxes here so pointers exposed to C remain valid until free().
pub struct Allocations {
    // Owned byte buffers for every allocated string.
    pub strings: Vec<Box<[u8]>>,
    // Owned boxed slices for Annotation arrays.
    pub annotations: Vec<Box<[Annotation]>>,
    // Owned boxed slices for arrays of FFIString
    pub ffi_strings_vecs: Vec<Box<[FFIString]>>,
    // Reserved: if you need to allocate arrays of TypeDecl on heap.
    pub type_decls: Vec<Box<[TypeDecl]>>,
    // Reserved: for single TypeDecl boxed allocations if needed.
    pub single_type_decls: Vec<Box<TypeDecl>>,
}

impl Allocations {
    pub fn new() -> Self {
        Self {
            strings: Vec::new(),
            annotations: Vec::new(),
            ffi_strings_vecs: Vec::new(),
            type_decls: Vec::new(),
            single_type_decls: Vec::new(),
        }
    }
}

impl Default for Allocations {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(C)]
pub struct IdlDoc {
    pub raw_ptr: Ptr,
    pub allocations: *mut Allocations,
}

#[repr(C)]
pub struct ParseResult {
    idl_doc: *mut IdlDoc,
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

fn create_parse_error(code: ErrorCode, details_str: impl Into<String>) -> *mut ParseResult {
    let details = CString::new(details_str.into()).unwrap();
    let result = ParseResult {
        error: Error {
            code,
            details: details.into_raw(),
        },
        idl_doc: ptr::null_mut(),
    };
    Box::into_raw(Box::new(result))
}

/// Parses an IDL source string into a Rust AST.
///
/// On failure, this function returns a `ParseResult` with error details.
///
/// # Safety
/// - `source_ptr` must be a valid, null-terminated UTF-8 string.
/// - The returned pointer must be freed using `free_parse_result` to avoid memory leaks.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn parse_idl(source_ptr: *const c_char) -> *mut ParseResult {
    if source_ptr.is_null() {
        return create_parse_error(ErrorCode::NullPtr, "Null pointer passed to parse_idl");
    }

    let source = unsafe {
        match CStr::from_ptr(source_ptr).to_str() {
            Ok(s) => s,
            Err(e) => {
                return create_parse_error(
                    ErrorCode::InvalidIDL,
                    format!("Invalid UTF-8 in source string: {e}"),
                );
            }
        }
    };

    // Parse AST
    let ast_doc = match crate::parse_idl(source) {
        Ok(doc) => doc,
        Err(e) => {
            return create_parse_error(ErrorCode::ParseError, format!("Failed to parse IDL: {e}"));
        }
    };

    // Prepare allocations manager (owned, will be freed in free_idl_doc)
    let allocations_box = Box::new(Allocations::new());
    let allocations_ptr = Box::into_raw(allocations_box);

    // Leak ast_doc but keep its pointer inside IdlDoc.raw_ptr.
    // We'''ll reconstruct and drop it in free_idl_doc.
    let boxed_ast = Box::new(ast_doc);
    let raw_ast_ptr = Box::into_raw(boxed_ast);

    let ffi_doc = IdlDoc {
        raw_ptr: Ptr(raw_ast_ptr as *const ()),
        allocations: allocations_ptr,
    };

    let result = ParseResult {
        error: Error {
            code: ErrorCode::Ok,
            details: ptr::null(),
        },
        idl_doc: Box::into_raw(Box::new(ffi_doc)),
    };

    Box::into_raw(Box::new(result))
}

/// Frees the memory for a `ParseResult` allocated by `parse_idl`.
///
/// # Safety
/// - `result_ptr` must be a pointer returned by a successful call to `parse_idl`.
/// - Passing a `NULL` pointer is safe (it will be a no-op).
/// - Do not use the pointer after it has been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn free_parse_result(result_ptr: *mut ParseResult) {
    if result_ptr.is_null() {
        return;
    }

    let result = unsafe { Box::from_raw(result_ptr) };

    if !result.idl_doc.is_null() {
        let ffi_doc = unsafe { Box::from_raw(result.idl_doc) };

        if !ffi_doc.raw_ptr.0.is_null() {
            let ast_box: *mut ast::IdlDoc = ffi_doc.raw_ptr.0 as *mut ast::IdlDoc;
            if !ast_box.is_null() {
                let _ = unsafe { Box::from_raw(ast_box) };
            }
        }

        if !ffi_doc.allocations.is_null() {
            let allocs = unsafe { Box::from_raw(ffi_doc.allocations) };
            drop(allocs);
        }
    }

    if result.error.code != ErrorCode::Ok && !result.error.details.is_null() {
        let details = unsafe { CString::from_raw(result.error.details as *mut _) };
        drop(details);
    }
}

/// A type-erased, opaque pointer to a node in the Rust AST.
///
/// The consumer of the FFI should not attempt to dereference this pointer directly.
/// It is used to maintain context and pass references to Rust objects across the FFI boundary.
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

// Helper function to allocate a string and store it in Allocations.
// We clone bytes into a Box<[u8]> so pointer remains valid.
fn allocate_string(s: &String, allocations: &mut Allocations) -> FFIString {
    let boxed_slice: Box<[u8]> = s.as_bytes().to_vec().into_boxed_slice();
    let ffi_string = FFIString {
        ptr: boxed_slice.as_ptr(),
        len: boxed_slice.len() as u32,
    };
    allocations.strings.push(boxed_slice);
    ffi_string
}

// Helper function to allocate a vector of FFIStrings and store it in Allocations.
fn allocate_ffi_string_vec(
    vec: &[String],
    allocations: &mut Allocations,
) -> (*const FFIString, u32) {
    if vec.is_empty() {
        return (ptr::null(), 0);
    }
    let mut temp: Vec<FFIString> = Vec::with_capacity(vec.len());
    for s in vec.iter() {
        let f = allocate_string(s, allocations);
        temp.push(f);
    }
    let boxed_slice = temp.into_boxed_slice();
    let ptr_out = boxed_slice.as_ptr();
    let len = boxed_slice.len() as u32;
    allocations.ffi_strings_vecs.push(boxed_slice);
    (ptr_out, len)
}

// Helper function to allocate a vector of Annotations and store it in Allocations.
fn allocate_annotation_vec(
    vec: &[(String, Option<String>)],
    allocations: &mut Allocations,
) -> (*const Annotation, u32) {
    if vec.is_empty() {
        return (ptr::null(), 0);
    }
    let mut temp: Vec<Annotation> = Vec::with_capacity(vec.len());
    for (name, value) in vec.iter() {
        let name_ffi = allocate_string(name, allocations);
        let (value_ptr, value_len, has_value) = if let Some(v) = value {
            let val_ffi = allocate_string(v, allocations);
            (val_ffi.ptr, val_ffi.len, true)
        } else {
            (ptr::null(), 0, false)
        };
        temp.push(Annotation {
            name_ptr: name_ffi.ptr,
            name_len: name_ffi.len,
            value_ptr,
            value_len,
            has_value,
        });
    }
    let boxed_slice = temp.into_boxed_slice();
    let ptr_out = boxed_slice.as_ptr();
    let len = boxed_slice.len() as u32;
    allocations.annotations.push(boxed_slice);
    (ptr_out, len)
}

/// FFI-safe representation of a [crate::ast::ProgramUnit].
#[repr(C)]
pub struct ProgramUnit {
    /// Opaque pointer to the original Rust AST node.
    pub raw_ptr: Ptr,
    /// Pointer to the UTF-8 encoded name of the program.
    pub name_ptr: *const u8,
    /// Length of the name string in bytes.
    pub name_len: u32,
    pub docs_ptr: *const FFIString,
    pub docs_len: u32,
    pub annotations_ptr: *const Annotation,
    pub annotations_len: u32,
}

impl ProgramUnit {
    pub fn from_ast(program_unit: &ast::ProgramUnit, allocations: &mut Allocations) -> Self {
        let name_ffi = allocate_string(&program_unit.name, allocations);
        let (docs_ptr, docs_len) = allocate_ffi_string_vec(&program_unit.docs, allocations);
        let (annotations_ptr, annotations_len) =
            allocate_annotation_vec(&program_unit.annotations, allocations);

        ProgramUnit {
            raw_ptr: Ptr::from(program_unit),
            name_ptr: name_ffi.ptr,
            name_len: name_ffi.len,
            docs_ptr,
            docs_len,
            annotations_ptr,
            annotations_len,
        }
    }
}

/// FFI-safe representation of a [crate::ast::ServiceUnit].
#[repr(C)]
pub struct ServiceUnit {
    pub raw_ptr: Ptr,
    pub name_ptr: *const u8,
    pub name_len: u32,
    pub extends_ptr: *const FFIString,
    pub extends_len: u32,
    pub docs_ptr: *const FFIString,
    pub docs_len: u32,
    pub annotations_ptr: *const Annotation,
    pub annotations_len: u32,
}

impl ServiceUnit {
    pub fn from_ast(service_unit: &ast::ServiceUnit, allocations: &mut Allocations) -> Self {
        let name_ffi = allocate_string(&service_unit.name, allocations);
        let (extends_ptr, extends_len) =
            allocate_ffi_string_vec(&service_unit.extends, allocations);
        let (docs_ptr, docs_len) = allocate_ffi_string_vec(&service_unit.docs, allocations);
        let (annotations_ptr, annotations_len) =
            allocate_annotation_vec(&service_unit.annotations, allocations);

        ServiceUnit {
            raw_ptr: Ptr::from(service_unit),
            name_ptr: name_ffi.ptr,
            name_len: name_ffi.len,
            extends_ptr,
            extends_len,
            docs_ptr,
            docs_len,
            annotations_ptr,
            annotations_len,
        }
    }
}

/// FFI-safe representation of a [crate::ast::CtorFunc].
#[repr(C)]
pub struct CtorFunc {
    pub raw_ptr: Ptr,
    pub name_ptr: *const u8,
    pub name_len: u32,
    pub docs_ptr: *const FFIString,
    pub docs_len: u32,
    pub annotations_ptr: *const Annotation,
    pub annotations_len: u32,
}

impl CtorFunc {
    pub fn from_ast(ctor_func: &ast::CtorFunc, allocations: &mut Allocations) -> Self {
        let name_ffi = allocate_string(&ctor_func.name, allocations);
        let (docs_ptr, docs_len) = allocate_ffi_string_vec(&ctor_func.docs, allocations);
        let (annotations_ptr, annotations_len) =
            allocate_annotation_vec(&ctor_func.annotations, allocations);

        CtorFunc {
            raw_ptr: Ptr::from(ctor_func),
            name_ptr: name_ffi.ptr,
            name_len: name_ffi.len,
            docs_ptr,
            docs_len,
            annotations_ptr,
            annotations_len,
        }
    }
}

/// FFI-safe representation of a [crate::ast::FuncParam].
#[repr(C)]
pub struct FuncParam {
    pub raw_ptr: Ptr,
    pub name_ptr: *const u8,
    pub name_len: u32,
}

impl FuncParam {
    pub fn from_ast(func_param: &ast::FuncParam, allocations: &mut Allocations) -> Self {
        let name_ffi = allocate_string(&func_param.name, allocations);
        FuncParam {
            raw_ptr: Ptr::from(func_param),
            name_ptr: name_ffi.ptr,
            name_len: name_ffi.len,
        }
    }
}

/// FFI-safe representation of a [crate::ast::Type].
#[repr(C)]
pub struct Type {
    pub raw_ptr: Ptr,
    pub name_ptr: *const u8,
    pub name_len: u32,
    pub docs_ptr: *const FFIString,
    pub docs_len: u32,
    pub annotations_ptr: *const Annotation,
    pub annotations_len: u32,
}

impl Type {
    pub fn from_ast(ty: &ast::Type, allocations: &mut Allocations) -> Self {
        let name_ffi = allocate_string(&ty.name, allocations);
        let (docs_ptr, docs_len) = allocate_ffi_string_vec(&ty.docs, allocations);
        let (annotations_ptr, annotations_len) =
            allocate_annotation_vec(&ty.annotations, allocations);

        Type {
            raw_ptr: Ptr::from(ty),
            name_ptr: name_ffi.ptr,
            name_len: name_ffi.len,
            docs_ptr,
            docs_len,
            annotations_ptr,
            annotations_len,
        }
    }
}

/// FFI-safe representation of a [crate::ast::TypeDecl].
#[repr(C)]
pub struct TypeDecl {
    pub raw_ptr: Ptr,
}

impl TypeDecl {
    pub fn from_ast(type_decl: &ast::TypeDecl, _allocations: &mut Allocations) -> Self {
        TypeDecl {
            raw_ptr: Ptr::from(type_decl),
        }
    }
}

/// FFI-safe representation of a [crate::ast::FunctionKind].
#[repr(C)]
pub enum FunctionKind {
    Query,
    Command,
}

/// FFI-safe representation of a [crate::ast::ServiceFunc].
#[repr(C)]
pub struct ServiceFunc {
    pub raw_ptr: Ptr,
    pub name_ptr: *const u8,
    pub name_len: u32,
    pub kind: FunctionKind,
    pub docs_ptr: *const FFIString,
    pub docs_len: u32,
    pub annotations_ptr: *const Annotation,
    pub annotations_len: u32,
}

impl ServiceFunc {
    pub fn from_ast(service_func: &ast::ServiceFunc, allocations: &mut Allocations) -> Self {
        let name_ffi = allocate_string(&service_func.name, allocations);
        let (docs_ptr, docs_len) = allocate_ffi_string_vec(&service_func.docs, allocations);
        let (annotations_ptr, annotations_len) =
            allocate_annotation_vec(&service_func.annotations, allocations);

        ServiceFunc {
            raw_ptr: Ptr::from(service_func),
            name_ptr: name_ffi.ptr,
            name_len: name_ffi.len,
            kind: match service_func.kind {
                ast::FunctionKind::Query => FunctionKind::Query,
                ast::FunctionKind::Command => FunctionKind::Command,
            },
            docs_ptr,
            docs_len,
            annotations_ptr,
            annotations_len,
        }
    }
}

/// FFI-safe representation of a [crate::ast::ServiceEvent].
#[repr(C)]
pub struct ServiceEvent {
    pub raw_ptr: Ptr,
    pub name_ptr: *const u8,
    pub name_len: u32,
    pub docs_ptr: *const FFIString,
    pub docs_len: u32,
    pub annotations_ptr: *const Annotation,
    pub annotations_len: u32,
}

impl ServiceEvent {
    pub fn from_ast(service_event: &ast::ServiceEvent, allocations: &mut Allocations) -> Self {
        let name_ffi = allocate_string(&service_event.name, allocations);
        let (docs_ptr, docs_len) = allocate_ffi_string_vec(&service_event.docs, allocations);
        let (annotations_ptr, annotations_len) =
            allocate_annotation_vec(&service_event.annotations, allocations);

        ServiceEvent {
            raw_ptr: Ptr::from(service_event),
            name_ptr: name_ffi.ptr,
            name_len: name_ffi.len,
            docs_ptr,
            docs_len,
            annotations_ptr,
            annotations_len,
        }
    }
}

/// FFI-safe representation of a [crate::ast::StructDef].
#[repr(C)]
pub struct StructDef {
    pub raw_ptr: Ptr,
}

impl StructDef {
    pub fn from_ast(struct_def: &ast::StructDef, _allocations: &mut Allocations) -> Self {
        StructDef {
            raw_ptr: Ptr::from(struct_def),
        }
    }
}

/// FFI-safe representation of a [crate::ast::StructField].
#[repr(C)]
pub struct StructField {
    pub raw_ptr: Ptr,
    pub name_ptr: *const u8,
    pub name_len: u32,
    pub docs_ptr: *const FFIString,
    pub docs_len: u32,
    pub annotations_ptr: *const Annotation,
    pub annotations_len: u32,
}

impl StructField {
    pub fn from_ast(struct_field: &ast::StructField, allocations: &mut Allocations) -> Self {
        let (name_ptr, name_len) = if let Some(name) = &struct_field.name {
            let name_ffi = allocate_string(name, allocations);
            (name_ffi.ptr, name_ffi.len)
        } else {
            (ptr::null(), 0)
        };

        let (docs_ptr, docs_len) = allocate_ffi_string_vec(&struct_field.docs, allocations);
        let (annotations_ptr, annotations_len) =
            allocate_annotation_vec(&struct_field.annotations, allocations);

        StructField {
            raw_ptr: Ptr::from(struct_field),
            name_ptr,
            name_len,
            docs_ptr,
            docs_len,
            annotations_ptr,
            annotations_len,
        }
    }
}

/// FFI-safe representation of a [crate::ast::EnumDef].
#[repr(C)]
pub struct EnumDef {
    pub raw_ptr: Ptr,
}

impl EnumDef {
    pub fn from_ast(enum_def: &ast::EnumDef, _allocations: &mut Allocations) -> Self {
        EnumDef {
            raw_ptr: Ptr::from(enum_def),
        }
    }
}

/// FFI-safe representation of a [crate::ast::EnumVariant].
#[repr(C)]
pub struct EnumVariant {
    pub raw_ptr: Ptr,
    pub name_ptr: *const u8,
    pub name_len: u32,
    pub docs_ptr: *const FFIString,
    pub docs_len: u32,
    pub annotations_ptr: *const Annotation,
    pub annotations_len: u32,
}

impl EnumVariant {
    pub fn from_ast(enum_variant: &ast::EnumVariant, allocations: &mut Allocations) -> Self {
        let name_ffi = allocate_string(&enum_variant.name, allocations);
        let (docs_ptr, docs_len) = allocate_ffi_string_vec(&enum_variant.docs, allocations);
        let (annotations_ptr, annotations_len) =
            allocate_annotation_vec(&enum_variant.annotations, allocations);

        EnumVariant {
            raw_ptr: Ptr::from(enum_variant),
            name_ptr: name_ffi.ptr,
            name_len: name_ffi.len,
            docs_ptr,
            docs_len,
            annotations_ptr,
            annotations_len,
        }
    }
}

/// FFI-safe representation of a [crate::ast::ServiceExpo].
#[repr(C)]
pub struct ServiceExpo {
    pub raw_ptr: Ptr,
    pub name_ptr: *const u8,
    pub name_len: u32,
    pub route_ptr: *const u8,
    pub route_len: u32,
    pub docs_ptr: *const FFIString,
    pub docs_len: u32,
    pub annotations_ptr: *const Annotation,
    pub annotations_len: u32,
}

impl ServiceExpo {
    pub fn from_ast(service_expo: &ast::ServiceExpo, allocations: &mut Allocations) -> Self {
        let name_ffi = allocate_string(&service_expo.name, allocations);
        let (route_ptr, route_len) = if let Some(route) = &service_expo.route {
            let route_ffi = allocate_string(route, allocations);
            (route_ffi.ptr, route_ffi.len)
        } else {
            (ptr::null(), 0)
        };
        let (docs_ptr, docs_len) = allocate_ffi_string_vec(&service_expo.docs, allocations);
        let (annotations_ptr, annotations_len) =
            allocate_annotation_vec(&service_expo.annotations, allocations);

        ServiceExpo {
            raw_ptr: Ptr::from(service_expo),
            name_ptr: name_ffi.ptr,
            name_len: name_ffi.len,
            route_ptr,
            route_len,
            docs_ptr,
            docs_len,
            annotations_ptr,
            annotations_len,
        }
    }
}

/// FFI-safe representation of a [crate::ast::TypeParameter].
#[repr(C)]
pub struct TypeParameter {
    pub raw_ptr: Ptr,
    pub name_ptr: *const u8,
    pub name_len: u32,
}

impl TypeParameter {
    pub fn from_ast(type_parameter: &ast::TypeParameter, allocations: &mut Allocations) -> Self {
        let name_ffi = allocate_string(&type_parameter.name, allocations);
        TypeParameter {
            raw_ptr: Ptr::from(type_parameter),
            name_ptr: name_ffi.ptr,
            name_len: name_ffi.len,
        }
    }
}

/// FFI-safe representation of a [crate::ast::TypeDef].
#[repr(C)]
pub struct TypeDef {
    pub raw_ptr: Ptr,
}

impl TypeDef {
    pub fn from_ast(type_def: &ast::TypeDef, _allocations: &mut Allocations) -> Self {
        TypeDef {
            raw_ptr: Ptr::from(type_def),
        }
    }
}
