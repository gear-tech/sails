#include <assert.h>

#include "utils.h"

// --- Global flags to check if visitors were called ---
int globals_visited = 0;
int program_unit_visited = 0;
int ctor_func_visited = 0;
int type_visited = 0;
int struct_def_visited = 0;
int struct_field_visited = 0;
int service_unit_visited = 0;
int service_expo_visited = 0;

// --- Visitor Callbacks ---

void c_visit_globals(const void *context, const Annotation *globals, uint32_t len) {
  printf("C: visit_globals called with %u annotations\n", len);
  globals_visited = 1;
}

void c_visit_program_unit(const void *context, const ProgramUnit *program) {
  printf("C: visit_program_unit called\n");
  program_unit_visited = 1;
  const Visitor *visitor = (const Visitor *)context;
  accept_program_unit(program, context, visitor);
}

void c_visit_ctor_func(const void *context, const CtorFunc *ctor) {
  printf("C: visit_ctor_func called\n");
  ctor_func_visited = 1;
}

void c_visit_type(const void *context, const Type *ty) {
  printf("C: visit_type called\n");
  type_visited = 1;
}

void c_visit_struct_def(const void *context, const StructDef *def) {
  printf("C: visit_struct_def called\n");
  struct_def_visited = 1;
}

void c_visit_struct_field(const void *context, const StructField *field) {
  printf("C: visit_struct_field called\n");
  struct_field_visited = 1;
}

void c_visit_service_unit(const void *context, const ServiceUnit *service) {
  printf("C: visit_service_unit called\n");
  service_unit_visited = 1;
}

void c_visit_service_expo(const void *context,
                                  const ServiceExpo *service_item) {
  printf("C: visit_service_expo called\n");
  service_expo_visited = 1;
}

void c_visit_named_type_decl(const void *context, const uint8_t *path,
                               uint32_t path_len, const TypeDecl *generics_ptr,
                               uint32_t generics_len) {
  printf("C: visit_named_type_decl called. Path: %.*s, Generics len: %u\n",
         path_len, path, generics_len);
}

int main() {
  const char *idl_source =
      "program MyProgram {\n"
      "    constructors {\n"
      "        NewCtor(param1: u32);\n"
      "    }\n"
      "}";

  ParseResult *result = parse_idl(idl_source);

  if (result == NULL) {
    fprintf(stderr, "Failed to parse IDL (result is null)\n");
    return 1;
  }

  if (result->error.code != Ok) {
    fprintf(stderr, "Failed to parse IDL: %s\n", result->error.details);
    free_parse_result(result);
    return 1;
  }

  assert(result->idl_doc != NULL);
  IdlDoc *doc = result->idl_doc;

  Visitor my_visitor = {
      .visit_globals = c_visit_globals,
      .visit_program_unit = c_visit_program_unit,
      .visit_ctor_func = c_visit_ctor_func,
      .visit_type =
          (void (*)(const void *,
                    const Type *))unexpected_ffi_call,  // Should not be called
      .visit_struct_def = (void (*)(const void *, const StructDef *))
          unexpected_ffi_call,  // Should not be called
      .visit_struct_field = (void (*)(const void *, const StructField *))
          unexpected_ffi_call,  // Should not be called
      .visit_service_unit = (void (*)(const void *, const ServiceUnit *))
          unexpected_ffi_call,  // Should not be called
      .visit_service_expo =
          (void (*)(const void *, const ServiceExpo *))
              unexpected_ffi_call,  // Should not be called
      // Initialize other function pointers to NULL if not implemented
      .visit_slice_type_decl =
          (void (*)(const void *, const TypeDecl *))unexpected_ffi_call,
      .visit_array_type_decl =
          (void (*)(const void *, const TypeDecl *,
                    uint32_t))unexpected_ffi_call_extra_args,
      .visit_tuple_type_decl = (void (*)(const void *, const TypeDecl *,
                    uint32_t))unexpected_ffi_call_extra_args,
      .visit_primitive_type =
          (void (*)(const void *, uint8_t))unexpected_ffi_call_extra_args,
      .visit_named_type_decl = c_visit_named_type_decl,
      .visit_service_func =
          (void (*)(const void *, const ServiceFunc *))unexpected_ffi_call,
      .visit_service_event =
          (void (*)(const void *, const ServiceEvent *))unexpected_ffi_call,
      .visit_enum_def =
          (void (*)(const void *, const EnumDef *))unexpected_ffi_call,
      .visit_enum_variant =
          (void (*)(const void *, const EnumVariant *))unexpected_ffi_call,
      .visit_type_parameter =
          (void (*)(const void *, const TypeParameter *))unexpected_ffi_call,
      .visit_type_def =
          (void (*)(const void *, const TypeDef *))unexpected_ffi_call,
  };

  // Pass the visitor as context so callbacks can continue the traversal chain.
  void *context = &my_visitor;

  ErrorCode visitor_result = accept_idl_doc(doc, context, &my_visitor);

  if (visitor_result != Ok) {
    fprintf(stderr, "Visitor traversal failed with error code: %d\n",
            visitor_result);
    free_parse_result(result);
    return 1;
  }

  printf("Visitor traversal completed.\n");

  // Assertions to check if visitors were called
  assert(globals_visited == 1 &&
         "visit_globals should have been called once");
  assert(program_unit_visited == 1 &&
         "visit_program_unit should have been called once");
  // TODO: check it
  assert(ctor_func_visited == 1 &&
         "visit_ctor_func should have been called once");

  // Ensure other visitors were NOT called
  assert(type_visited == 0 && "visit_type should NOT have been called");
  assert(struct_def_visited == 0 &&
         "visit_struct_def should NOT have been called");
  assert(struct_field_visited == 0 &&
         "visit_struct_field should NOT have been called");
  assert(service_unit_visited == 0 &&
         "visit_service_unit should NOT have been called");
  assert(service_expo_visited == 0 &&
         "visit_service_expo should NOT have been called");

  free_parse_result(result);
  printf("ParseResult freed.\n");

  return 0;
}