use insta::assert_snapshot;
use sails_sol_gen::generate_solidity_contract;

const SIMPLE_IDL: &str = r#"
constructor {
    CreatePrg : ();
};

service Svc1 {
    DoThis : (p1: u32, p2: str) -> u32;
};"#;

const IDL_W_EVENTS: &str = r#"
constructor {
  Create : ();
};
service Svc1 {
  DoThis : (p1: u32, p2: str) -> u32;
  events {
    DoThisEvent: struct { p1: u32, p2: str };
    DoThisEvent2: struct { u32, str };
  }
};"#;

#[test]
fn test_generate_simple_contract() {
    let contract = generate_solidity_contract(SIMPLE_IDL, "TestContract");

    assert!(contract.is_ok());
    assert_snapshot!(String::from_utf8(contract.unwrap()).unwrap());
}

#[test]
fn test_generate_contract_w_events() {
    let contract = generate_solidity_contract(IDL_W_EVENTS, "TestContract");

    assert!(contract.is_ok());
    assert_snapshot!(String::from_utf8(contract.unwrap()).unwrap());
}
