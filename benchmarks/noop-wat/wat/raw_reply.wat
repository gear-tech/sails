(module
  (import "env" "memory" (memory 1))
  (import "env" "gr_reply" (func $gr_reply (param i32 i32 i32 i32)))

  (data (i32.const 256) "\01")

  (func $init)

  (func $handle
    (call $gr_reply
      (i32.const 256)
      (i32.const 1)
      (i32.const -1)
      (i32.const 272)))

  (export "init" (func $init))
  (export "handle" (func $handle)))
