#![cfg(feature = "ethexe")]

mod program_basic;

#[test]
fn program_basic() {
    use program_basic::MyProgram;
    let _prg = MyProgram::new(42);
    let _prg = MyProgram::new_forty_two().unwrap();
}
