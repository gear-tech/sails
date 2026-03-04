use sails_sol_client_gen::ClientGenerator;

#[test]
fn generated_sol_client_has_expected_selectors_and_abi_shape() {
    let idl = include_str!("../ethapp.idl");

    let code = ClientGenerator::from_idl(idl)
        .with_sails_crate("sails_rs")
        .generate()
        .expect("generate solidity rust client");

    assert!(code.contains("io_struct_sol_impl!(CreatePrg"));
    assert!(code.contains("\"createPrg\""));
    assert!(code.contains("io_struct_sol_impl!("));
    assert!(code.contains("\"svc1\""));
    assert!(code.contains("\"DoThis\""));
    assert!(code.contains("SolValue::abi_encode_sequence"));
    assert!(code.contains("SolValue::abi_decode"));
}

