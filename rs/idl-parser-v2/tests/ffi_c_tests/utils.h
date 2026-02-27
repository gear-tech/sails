#ifndef FFI_C_TESTS_UTILS_H
#define FFI_C_TESTS_UTILS_H

#include <stdint.h> // For uint32_t, uint8_t
#include <stdio.h>  
#include <stdlib.h> 


#include "idl_parser_v2_ffi.h"

#ifdef __cplusplus
extern "C" {
#endif

/**
 * @brief Reads the content of a file into a dynamically allocated string.
 *        Exits with an error if the file cannot be opened or memory allocation fails.
 *
 * @param filename The path to the file to read.
 * @return A pointer to the null-terminated string containing the file content.
 *         The caller is responsible for freeing this memory.
 */
char *read_file_to_string(const char *filename);

/**
 * @brief Generic callback for FFI visitor functions that should not be called.
 *        Prints an error message and exits.
 *
 * @param context The visitor context.
 * @param ptr A pointer to the AST node.
 */
void unexpected_ffi_call(const void *context, const void *ptr);

/**
 * @brief Generic callback for FFI visitor functions with extra arguments that should not be called.
 *        Prints an error message and exits.
 *
 * @param context The visitor context.
 * @param ptr A pointer to the AST node.
 * @param ... Variable arguments for extra parameters.
 */
void unexpected_ffi_call_extra_args(const void *context, const void *ptr, ...);

#ifdef __cplusplus
}
#endif

#endif // FFI_C_TESTS_UTILS_H
