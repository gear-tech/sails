#include <assert.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "utils.h"

int main() {
  printf("Running C FFI test for null visitor callbacks...\n");

  const char* idl_source = "program MyProgram { constructors { new(); } }";
  ParseResult* result = parse_idl(idl_source);

  if (result == NULL) {
    fprintf(stderr, "ERROR: Failed to parse IDL (result is null)\n");
    return 1;
  }

  if (result->error.code != Ok) {
    fprintf(stderr, "ERROR: Failed to parse IDL: %s\n", result->error.details);
    free_parse_result(result);
    return 1;
  }

  assert(result->idl_doc != NULL);
  IdlDoc* doc_ptr = result->idl_doc;

  Visitor null_visitor = {
      .visit_globals = NULL,
      .visit_program_unit = NULL,
      .visit_service_unit = NULL,
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
      .visit_service_expo = NULL,
      .visit_type_parameter = NULL,
      .visit_type_def = NULL,
  };

  ErrorCode visitor_result = accept_idl_doc(doc_ptr, NULL, &null_visitor);

  free_parse_result(result);

  assert(visitor_result == Ok && "Expected ErrorCode::Ok from accept_idl_doc");

  printf("C FFI test for null visitor callbacks PASSED.\n");

  return 0;
}
