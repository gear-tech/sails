use std::{env, fs, path::PathBuf};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is set"));

    let cases = [
        ("raw_reply", include_str!("wat/raw_reply.wat")),
        ("sails_wire_reply", include_str!("wat/sails_wire_reply.wat")),
    ];

    for (name, wat_text) in cases {
        let wasm = wat::parse_str(wat_text).expect("WAT noop fixture compiles");
        fs::write(out_dir.join(format!("{name}.wasm")), wasm).expect("WAT noop fixture writes");
    }
}
