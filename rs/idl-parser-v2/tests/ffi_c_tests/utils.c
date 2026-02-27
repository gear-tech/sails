#include "utils.h"

#include <assert.h>
#include <string.h>

char *read_file_to_string(const char *filename) {
  FILE *file = fopen(filename, "rb");
  if (file == NULL) {
    fprintf(stderr, "ERROR: Could not open file %s\n", filename);
    exit(1);
  }

  fseek(file, 0, SEEK_END);
  long length = ftell(file);
  fseek(file, 0, SEEK_SET);

  char *buffer = (char *)malloc(length + 1);
  if (buffer == NULL) {
    fprintf(stderr, "ERROR: Memory allocation failed for file %s\n", filename);
    fclose(file);
    exit(1);
  }

  fread(buffer, 1, length, file);
  buffer[length] = '\0';

  fclose(file);
  return buffer;
}

void unexpected_ffi_call(const void *context, const void *ptr) {
  (void)context;
  (void)ptr;
  fprintf(stderr, "ERROR: An unexpected FFI callback was called!\n");
  exit(1);
}

void unexpected_ffi_call_extra_args(const void *context, const void *ptr, ...) {
  (void)context;
  (void)ptr;
  fprintf(stderr,
          "ERROR: An unexpected FFI callback with extra args was called!\n");
  exit(1);
}
