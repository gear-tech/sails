#include <assert.h>

#include "utils.h"

// --- Globals for Counters ---
static int count_globals = 0;
static int count_program_unit = 0;
static int count_service_unit = 0;
static int count_ctor_func = 0;
static int count_func_param = 0;
static int count_type = 0;
static int count_slice_type_decl = 0;
static int count_array_type_decl = 0;
static int count_tuple_type_decl = 0;
static int count_named_type_decl = 0;
static int count_primitive_type = 0;
static int count_service_func = 0;
static int count_service_event = 0;
static int count_struct_def = 0;
static int count_struct_field = 0;
static int count_enum_def = 0;
static int count_enum_variant = 0;
static int count_service_expo = 0;
static int count_type_parameter = 0;
static int count_type_def = 0;

// --- Visitor Callback Implementations ---

void cb_visit_globals(const void *context, const Annotation *globals, uint32_t len) {
  count_globals++;
}

void cb_visit_program_unit(const void *context, const ProgramUnit *node) {
  count_program_unit++;
  accept_program_unit(node, context, (const Visitor *)context);
}

void cb_visit_service_unit(const void *context, const ServiceUnit *node) {
  count_service_unit++;
  accept_service_unit(node, context, (const Visitor *)context);
}

void cb_visit_ctor_func(const void *context, const CtorFunc *node) {
  count_ctor_func++;
  accept_ctor_func(node, context, (const Visitor *)context);
}

void cb_visit_func_param(const void *context, const FuncParam *node) {
  count_func_param++;
  accept_func_param(node, context, (const Visitor *)context);
}

void cb_visit_type(const void *context, const Type *node) {
  count_type++;
  accept_type(node, context, (const Visitor *)context);
}

void cb_visit_slice_type_decl(const void *context, const TypeDecl *item_ty) {
  count_slice_type_decl++;
  accept_type_decl(item_ty, context, (const Visitor *)context);
}

void cb_visit_array_type_decl(const void *context, const TypeDecl *item_ty,
                              uint32_t len) {
  count_array_type_decl++;
  accept_type_decl(item_ty, context, (const Visitor *)context);
}

void cb_visit_tuple_type_decl(const void *context, const TypeDecl *items,
                              uint32_t len) {
  count_tuple_type_decl++;
  for (uint32_t i = 0; i < len; i++) {
    const TypeDecl *item = (const TypeDecl *)((const char *)items + i * sizeof(void *));
    accept_type_decl(item, context, (const Visitor *)context);
  }
}

void cb_visit_primitive_type(const void *context, uint8_t primitive) {
  count_primitive_type++;
  // Leaf node, no accept call
}

void cb_visit_named_type_decl(const void *context, const uint8_t *path,
                                uint32_t path_len,
                                const TypeDecl *generics_ptr,
                                uint32_t generics_len) {
  count_named_type_decl++;
  for (uint32_t i = 0; i < generics_len; ++i) {
    const TypeDecl *generic = (const TypeDecl *)((const char *)generics_ptr + i * sizeof(void *));
    accept_type_decl(generic, context, (const Visitor *)context);
  }
}

void cb_visit_service_func(const void *context, const ServiceFunc *node) {
  count_service_func++;
  accept_service_func(node, context, (const Visitor *)context);
}

void cb_visit_service_event(const void *context, const ServiceEvent *node) {
  count_service_event++;
  accept_service_event(node, context, (const Visitor *)context);
}

void cb_visit_struct_def(const void *context, const StructDef *node) {
  count_struct_def++;
  accept_struct_def(node, context, (const Visitor *)context);
}

void cb_visit_struct_field(const void *context, const StructField *node) {
  count_struct_field++;
  accept_struct_field(node, context, (const Visitor *)context);
}

void cb_visit_enum_def(const void *context, const EnumDef *node) {
  count_enum_def++;
  accept_enum_def(node, context, (const Visitor *)context);
}

void cb_visit_enum_variant(const void *context, const EnumVariant *node) {
  count_enum_variant++;
  accept_enum_variant(node, context, (const Visitor *)context);
}

void cb_visit_service_expo(const void *context,
                                   const ServiceExpo *node) {
  count_service_expo++;
  accept_service_expo(node, context, (const Visitor *)context);
}

void cb_visit_type_parameter(const void *context, const TypeParameter *node) {
  count_type_parameter++;
  accept_type_parameter(node, context, (const Visitor *)context);
}

void cb_visit_type_def(const void *context, const TypeDef *node) {
  count_type_def++;
  accept_type_def(node, context, (const Visitor *)context);
}

// --- Main Test Logic ---

int main() {
  printf("Running Full Coverage C FFI test...\n");

  char *idl_source = read_file_to_string(IDL_FILE_PATH_FULL_COVERAGE);
  assert(idl_source != NULL && "Failed to read full_coverage.idl");

  ParseResult *result = parse_idl(idl_source);
  free(idl_source);

  assert(result != NULL && "parse_idl returned null");
  if (result->error.code != Ok) {
    fprintf(stderr, "PARSING FAILED: %s\n", result->error.details);
    free_parse_result(result);
    assert(0 && "Parsing failed, see error message above.");
    return 1;
  }
  assert(result->idl_doc != NULL && "parsed doc is null");

  Visitor full_visitor = {
      .visit_globals = cb_visit_globals,
      .visit_program_unit = cb_visit_program_unit,
      .visit_service_unit = cb_visit_service_unit,
      .visit_ctor_func = cb_visit_ctor_func,
      .visit_func_param = cb_visit_func_param,
      .visit_type = cb_visit_type,
      .visit_slice_type_decl = cb_visit_slice_type_decl,
      .visit_array_type_decl = cb_visit_array_type_decl,
      .visit_tuple_type_decl = cb_visit_tuple_type_decl,
      .visit_primitive_type = cb_visit_primitive_type,
      .visit_named_type_decl = cb_visit_named_type_decl,
      .visit_service_func = cb_visit_service_func,
      .visit_service_event = cb_visit_service_event,
      .visit_struct_def = cb_visit_struct_def,
      .visit_struct_field = cb_visit_struct_field,
      .visit_enum_def = cb_visit_enum_def,
      .visit_enum_variant = cb_visit_enum_variant,
      .visit_service_expo = cb_visit_service_expo,
      .visit_type_parameter = cb_visit_type_parameter,
      .visit_type_def = cb_visit_type_def,
  };

  // The context pointer will be our visitor itself.
  ErrorCode visit_result =
      accept_idl_doc(result->idl_doc, &full_visitor, &full_visitor);
  assert(visit_result == Ok && "accept_idl_doc failed");

  free_parse_result(result);

  printf("Final counts:\n");
  printf("  globals: %d\n", count_globals);
  printf("  program_unit: %d\n", count_program_unit);
  printf("  service_unit: %d\n", count_service_unit);
  printf("  ctor_func: %d\n", count_ctor_func);
  printf("  func_param: %d\n", count_func_param);
  printf("  type: %d\n", count_type);
  printf("  slice_type_decl: %d\n", count_slice_type_decl);
  printf("  array_type_decl: %d\n", count_array_type_decl);
  printf("  tuple_type_decl: %d\n", count_tuple_type_decl);
  printf("  named_type_decl: %d\n", count_named_type_decl);
  printf("  primitive_type: %d\n", count_primitive_type);
  printf("  service_func: %d\n", count_service_func);
  printf("  service_event: %d\n", count_service_event);
  printf("  struct_def: %d\n", count_struct_def);
  printf("  struct_field: %d\n", count_struct_field);
  printf("  enum_def: %d\n", count_enum_def);
  printf("  enum_variant: %d\n", count_enum_variant);
  printf("  service_expo: %d\n", count_service_expo);
  printf("  type_parameter: %d\n", count_type_parameter);
  printf("  type_def: %d\n", count_type_def);

  printf("Checking assertions...\n");
  assert(count_globals == 1);
  assert(count_program_unit == 1);
  assert(count_service_unit == 2);
  assert(count_ctor_func == 1);
  assert(count_func_param == 1);
  assert(count_type == 6);
  assert(count_slice_type_decl == 1);
  assert(count_array_type_decl == 1);
  assert(count_tuple_type_decl == 1);
  assert(count_named_type_decl == 5);
  assert(count_primitive_type == 22);
  assert(count_service_func == 3);
  assert(count_service_event == 3);
  assert(count_struct_def == 11);
  assert(count_struct_field == 17);
  assert(count_enum_def == 1);
  assert(count_enum_variant == 6);
  assert(count_service_expo == 2);
  assert(count_type_parameter == 1);
  assert(count_type_def == 6);

  int total_type_decls = count_slice_type_decl +
                         count_array_type_decl +
                         count_tuple_type_decl +
                         count_named_type_decl +
                         count_primitive_type;
  assert(total_type_decls == 30);

  printf("Full Coverage C FFI test PASSED.\n");
  return 0;
}