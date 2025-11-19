#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

// Forward declarations for opaque pointers
typedef struct IdlDoc IdlDoc;
typedef struct ProgramUnit ProgramUnit;
typedef struct ServiceUnit ServiceUnit;
typedef struct CtorFunc CtorFunc;
typedef struct FuncParam FuncParam;
typedef struct Type Type;
typedef struct TypeDecl TypeDecl;
typedef struct ServiceFunc ServiceFunc;
typedef struct ServiceEvent ServiceEvent;
typedef struct StructDef StructDef;
typedef struct StructField StructField;
typedef struct EnumDef EnumDef;
typedef struct EnumVariant EnumVariant;
typedef struct ServiceExpo ServiceExpo;
typedef struct TypeParameter TypeParameter;
typedef struct TypeDef TypeDef;
typedef struct ParseResult ParseResult;
typedef struct Error Error;

// ErrorCode enum (from ffi/ast/mod.rs)
typedef enum ErrorCode {
  Ok,
  InvalidIDL,
  ParseError,
  NullPtr,
} ErrorCode;

// New Error and ParseResult structs
typedef struct Error {
  ErrorCode code;
  const char *details;
} Error;

typedef struct ParseResult {
  IdlDoc *idl_doc;
  Error error;
} ParseResult;

// Visitor struct (from ffi/ast/visitor.rs)
// Note: All fields are function pointers. NULL means None.
typedef struct Visitor {
  void (*visit_program_unit)(const void *context, const ProgramUnit *program);
  void (*visit_service_unit)(const void *context, const ServiceUnit *service);
  void (*visit_ctor_func)(const void *context, const CtorFunc *ctor);
  void (*visit_func_param)(const void *context, const FuncParam *param);
  void (*visit_type)(const void *context, const Type *ty);
  void (*visit_slice_type_decl)(const void *context, const TypeDecl *item_ty);
  void (*visit_array_type_decl)(const void *context, const TypeDecl *item_ty,
                                uint32_t len);
  void (*visit_tuple_type_decl)(const void *context, const TypeDecl *items,
                                uint32_t items_len);
  void (*visit_primitive_type)(const void *context, uint8_t primitive);
  void (*visit_named_type_decl)(const void *context, const uint8_t *path,
                                  uint32_t path_len,
                                  const TypeDecl *generics_ptr,
                                  uint32_t generics_len);
  void (*visit_service_func)(const void *context, const ServiceFunc *func);
  void (*visit_service_event)(const void *context, const ServiceEvent *event);
  void (*visit_struct_def)(const void *context, const StructDef *def);
  void (*visit_struct_field)(const void *context, const StructField *field);
  void (*visit_enum_def)(const void *context, const EnumDef *def);
  void (*visit_enum_variant)(const void *context, const EnumVariant *variant);
  void (*visit_service_expo)(const void *context,
                                     const ServiceExpo *service_item);
  void (*visit_type_parameter)(const void *context,
                               const TypeParameter *type_param);
  void (*visit_type_def)(const void *context, const TypeDef *type_def);
} Visitor;

// FFI functions from ffi/ast/mod.rs and ffi/ast/visitor.rs
ErrorCode accept_idl_doc(const IdlDoc *doc, const void *context,
                         const Visitor *visitor);
ErrorCode accept_program_unit(const ProgramUnit *program, const void *context, const Visitor *visitor);
ErrorCode accept_service_unit(const ServiceUnit *service, const void *context, const Visitor *visitor);
ErrorCode accept_ctor_func(const CtorFunc *ctor, const void *context, const Visitor *visitor);
ErrorCode accept_func_param(const FuncParam *param, const void *context, const Visitor *visitor);
ErrorCode accept_type(const Type *ty, const void *context, const Visitor *visitor);
ErrorCode accept_service_func(const ServiceFunc *func, const void *context, const Visitor *visitor);
ErrorCode accept_service_event(const ServiceEvent *event, const void *context, const Visitor *visitor);
ErrorCode accept_struct_def(const StructDef *def, const void *context, const Visitor *visitor);
ErrorCode accept_struct_field(const StructField *field, const void *context, const Visitor *visitor);
ErrorCode accept_enum_def(const EnumDef *def, const void *context, const Visitor *visitor);
ErrorCode accept_enum_variant(const EnumVariant *variant, const void *context, const Visitor *visitor);
ErrorCode accept_service_expo(const ServiceExpo *service_item,
                                      const void *context,
                                      const Visitor *visitor);
ErrorCode accept_type_decl(const TypeDecl *type_decl, const void *context,
                           const Visitor *visitor);
ErrorCode accept_type_parameter(const TypeParameter *type_param,
                                const void *context, const Visitor *visitor);
ErrorCode accept_type_def(const TypeDef *type_def, const void *context,
                          const Visitor *visitor);
// Functions for parsing and error handling
ParseResult *parse_idl(const char *source_ptr);
void free_parse_result(ParseResult *result_ptr);