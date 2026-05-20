(module
  (import "env" "memory" (memory 1))
  (import "env" "gr_size" (func $gr_size (param i32)))
  (import "env" "gr_read" (func $gr_read (param i32 i32 i32 i32)))
  (import "env" "gr_reply" (func $gr_reply (param i32 i32 i32 i32)))

  ;; Sails v1 header for NoopSails.noop plus SCALE bool(true).
  ;; "GM", version=1, hlen=16, interface=5457cc30dd94ce29,
  ;; entry_id=0, route_id=1, reserved=0, bool=true.
  (data (i32.const 512) "\47\4d\01\10\54\57\cc\30\dd\94\ce\29\00\00\01\00\01")

  (func $init)

  (func $handle
    (call $gr_size (i32.const 256))
    (call $gr_read
      (i32.const 0)
      (i32.load (i32.const 256))
      (i32.const 320)
      (i32.const 288))
    (call $gr_reply
      (i32.const 512)
      (i32.const 17)
      (i32.const -1)
      (i32.const 272)))

  (export "init" (func $init))
  (export "handle" (func $handle)))
