mod gprogram_basic;

#[test]
fn gprogram_basic() {
    use gprogram_basic::MyProgram;
    let _prg = MyProgram::new(42);
}
