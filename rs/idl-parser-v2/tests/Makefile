# rs/idl-parser-v2/tests/Makefile

CRATE_DIR := $(shell dirname $(realpath $(lastword $(MAKEFILE_LIST))))/..
TARGET_DIR := $(CRATE_DIR)/target/debug
C_TEST_DIR := $(CRATE_DIR)/tests/ffi_c_tests
IDLS_DIR := $(CRATE_DIR)/tests/idls

# C source files
C_SRCS := $(wildcard $(C_TEST_DIR)/*.c)
# Object files
C_OBJS := $(patsubst $(C_TEST_DIR)/%.c, $(TARGET_DIR)/%.o, $(C_SRCS))

# Test executables
TEST_NULL_VISITOR := $(TARGET_DIR)/ffi_null_visitor_test
TEST_DEMO_VISITOR := $(TARGET_DIR)/ffi_demo_idl_visitor_test
TEST_CUSTOM := $(TARGET_DIR)/ffi_custom_test
TEST_FULL_COVERAGE := $(TARGET_DIR)/ffi_full_coverage_test

.PHONY: all clean run $(TARGET_DIR)/libsails_idl_parser_v2.so

all: $(TEST_NULL_VISITOR) $(TEST_DEMO_VISITOR) $(TEST_CUSTOM) $(TEST_FULL_COVERAGE)

run: all
	@echo "Running C FFI tests..."
	LD_LIBRARY_PATH="$(TARGET_DIR):$(LD_LIBRARY_PATH)" $(TEST_NULL_VISITOR)
	LD_LIBRARY_PATH="$(TARGET_DIR):$(LD_LIBRARY_PATH)" $(TEST_DEMO_VISITOR)
	LD_LIBRARY_PATH="$(TARGET_DIR):$(LD_LIBRARY_PATH)" $(TEST_CUSTOM)
	LD_LIBRARY_PATH="$(TARGET_DIR):$(LD_LIBRARY_PATH)" $(TEST_FULL_COVERAGE)
	@echo "C FFI tests completed."

# Rule to build the Rust library
$(TARGET_DIR)/libsails_idl_parser_v2.so:
	cargo clean -p sails-idl-parser-v2
	cargo build -p sails-idl-parser-v2 --target-dir "$(CRATE_DIR)/target" --lib --features=ffi

# Rule to compile C source files into object files
$(TARGET_DIR)/%.o: $(C_TEST_DIR)/%.c $(C_TEST_DIR)/utils.h $(TARGET_DIR)/libsails_idl_parser_v2.so
	gcc -I$(C_TEST_DIR) -c $< -o $@

$(TARGET_DIR)/test_demo_idl_visitor.o: $(C_TEST_DIR)/test_demo_idl_visitor.c $(C_TEST_DIR)/utils.h
	gcc -I$(C_TEST_DIR) -DIDL_FILE_PATH=\"$(IDLS_DIR)/demo.idl\" -c $< -o $@

$(TARGET_DIR)/test_full_coverage.o: $(C_TEST_DIR)/test_full_coverage.c $(C_TEST_DIR)/utils.h
	gcc -I$(C_TEST_DIR) -DIDL_FILE_PATH_FULL_COVERAGE=\"$(IDLS_DIR)/full_coverage.idl\" -c $< -o $@

# Rule to link test executables
$(TEST_NULL_VISITOR): $(TARGET_DIR)/test_null_visitor.o $(TARGET_DIR)/utils.o $(TARGET_DIR)/libsails_idl_parser_v2.so
	gcc $(TARGET_DIR)/test_null_visitor.o $(TARGET_DIR)/utils.o -L$(TARGET_DIR) -lsails_idl_parser_v2 -o $@

$(TEST_DEMO_VISITOR): $(TARGET_DIR)/test_demo_idl_visitor.o $(TARGET_DIR)/utils.o $(TARGET_DIR)/libsails_idl_parser_v2.so
	gcc $(TARGET_DIR)/test_demo_idl_visitor.o $(TARGET_DIR)/utils.o -DIDL_FILE_PATH=\"$(IDLS_DIR)/demo.idl\" -L$(TARGET_DIR) -lsails_idl_parser_v2 -o $@

$(TEST_CUSTOM): $(TARGET_DIR)/ffi_custom_test.o $(TARGET_DIR)/utils.o $(TARGET_DIR)/libsails_idl_parser_v2.so
	gcc $(TARGET_DIR)/ffi_custom_test.o $(TARGET_DIR)/utils.o -L$(TARGET_DIR) -lsails_idl_parser_v2 -o $@

$(TEST_FULL_COVERAGE): $(TARGET_DIR)/test_full_coverage.o $(TARGET_DIR)/utils.o $(TARGET_DIR)/libsails_idl_parser_v2.so
	gcc $(TARGET_DIR)/test_full_coverage.o $(TARGET_DIR)/utils.o -DIDL_FILE_PATH_FULL_COVERAGE=\"$(IDLS_DIR)/full_coverage.idl\" -L$(TARGET_DIR) -lsails_idl_parser_v2 -o $@

clean:
	rm -f $(TARGET_DIR)/*.o $(TARGET_DIR)/ffi_*_test

.PHONY: valgrind-full-coverage
valgrind-full-coverage: $(TEST_FULL_COVERAGE)
	@if ! command -v valgrind &> /dev/null; then \
		echo "Error: valgrind is not installed. Please install it to run this target."; \
		exit 1; \
	fi
	@echo "Running ffi_full_coverage_test with Valgrind..."
	LD_LIBRARY_PATH="$(TARGET_DIR):$(LD_LIBRARY_PATH)" \
	valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes --num-callers=50 \
	$(TEST_FULL_COVERAGE)
