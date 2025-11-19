#include <assert.h>

#include "utils.h"

static int c_visit_program_unit_calls = 0;
static int c_visit_service_unit_calls = 0;
static int c_visit_service_expo_calls = 0;

// C-side callback implementations
void c_visit_program_unit(const void *context, const ProgramUnit *program) {
  (void)program;
  c_visit_program_unit_calls++;
  // Continue traversal by calling the FFI accept function
  accept_program_unit(program, context, (const Visitor *)context);
}

void c_visit_service_unit(const void *context, const ServiceUnit *service) {
  (void)context;
  (void)service;
  c_visit_service_unit_calls++;
}

void c_visit_service_expo(const void *context,
                                  const ServiceExpo *service_item) {
  (void)context;
  (void)service_item;
  c_visit_service_expo_calls++;
}

// Function to read file content into a string

int main() {
  printf("Running C FFI test for demo IDL visitor callbacks...\n");

  char *idl_source = read_file_to_string(IDL_FILE_PATH);

  ParseResult *result = parse_idl(idl_source);

  free(idl_source);

  if (result == NULL) {
    fprintf(stderr, "ERROR: Failed to parse IDL (result is null)\n");
    return 1;
  }

  if (result->error.code != Ok) {
    fprintf(stderr, "ERROR: Failed to parse IDL: %s\n", result->error.details);
    free_parse_result(result);
    return 1;  // Fail test
  }

  assert(result->idl_doc != NULL);
  IdlDoc *doc_ptr = result->idl_doc;

  // Create a C Visitor struct with some callbacks implemented and others NULL
  Visitor partial_visitor = {
      .visit_program_unit = c_visit_program_unit,
      .visit_service_unit = c_visit_service_unit,
      // All other callbacks are NULL, expecting Rust to use its fallback logic
      .visit_ctor_func = NULL,
      .visit_func_param = NULL,
      .visit_type = NULL,
      .visit_slice_type_decl = NULL,
      .visit_array_type_decl = NULL,
      .visit_tuple_type_decl = NULL,
      .visit_primitive_type = NULL,
      .visit_named_type_decl = NULL,
      .visit_service_func = NULL,
      .visit_service_event = NULL,
      .visit_struct_def = NULL,
      .visit_struct_field = NULL,
      .visit_enum_def = NULL,
      .visit_enum_variant = NULL,
      .visit_service_expo = c_visit_service_expo,
  
      .visit_type_parameter = NULL,
      .visit_type_def = NULL,
  };

  // Call the Rust FFI function accept_idl_doc
  ErrorCode visitor_result =
      accept_idl_doc(doc_ptr, &partial_visitor, &partial_visitor);

  // Free the memory allocated by Rust
  free_parse_result(result);

  // Assert the result
  assert(visitor_result == Ok && "Expected ErrorCode::Ok from accept_idl_doc");

  // Assert that our C callbacks were called the expected number of times
  assert(c_visit_program_unit_calls == 1 &&
         "c_visit_program_unit should have been called once");
  assert(c_visit_service_unit_calls == 6 &&
         "c_visit_service_unit should have been called 6 times for top-level "
         "services");
  assert(c_visit_service_expo_calls == 6 &&
         "c_visit_service_expo should have been called 6 times");

  printf("C FFI test for demo IDL visitor callbacks PASSED.\n");

  return 0;
}
