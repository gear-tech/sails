mod program_basic;

#[test]
fn gprogram_basic() {
    use program_basic::MyProgram;
    let _prg = MyProgram::new(42);
}
