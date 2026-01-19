use insta::assert_snapshot;
use sails_sol_gen::generate_solidity_contract;

const SIMPLE_IDL: &str = r#"
program TestProgram {
    constructors {
        CreatePrg();
    }
    services {
        Svc1: Svc1
    }
}

service Svc1 {
    functions {
        DoThis(p1: u32, p2: String) -> u32;
    }
}
"#;

const IDL_W_EVENTS: &str = r#"
program TestProgram {
    constructors {
        Create();
    }
    services {
        Svc1: Svc1
    }
}

service Svc1 {
    functions {
        DoThis(p1: u32, p2: String) -> u32;
    }
    events {
        DoThisEvent { p1: u32, p2: String },
        DoThisEvent2(u32, String),
    }
}
"#;

#[test]
fn test_generate_simple_contract() {
    let contract = generate_solidity_contract(SIMPLE_IDL, "TestContract");

    assert!(
        contract.is_ok(),
        "Failed to generate contract: {:?}",
        contract.err()
    );
    assert_snapshot!(String::from_utf8(contract.unwrap().data).unwrap());
}

#[test]
fn test_generate_contract_w_events() {
    let contract = generate_solidity_contract(IDL_W_EVENTS, "TestContract");

    assert!(
        contract.is_ok(),
        "Failed to generate contract: {:?}",
        contract.err()
    );
    assert_snapshot!(String::from_utf8(contract.unwrap().data).unwrap());
}

const IDL_MIXED_INDEXED: &str = r#"
program TestProgram {
    constructors {
        Create();
    }
    services {
        Svc: Svc
    }
}

service Svc {
    events {
        MixedEvent {
            /// #[indexed]
            f1: u32,
            f2: String,
            /// #[indexed]
            f3: u128,
            f4: u128
        }
    }
}
"#;

#[test]
fn test_generate_contract_w_mixed_indexed_events() {
    let contract = generate_solidity_contract(IDL_MIXED_INDEXED, "TestContract");

    assert!(
        contract.is_ok(),
        "Failed to generate contract: {:?}",
        contract.err()
    );
    assert_snapshot!(String::from_utf8(contract.unwrap().data).unwrap());
}

const PAYABLE_IDL: &str = r#"
program TestProgram {
    constructors {
        /// #[payable]
        Create();
    }
    services {
        MyService
    }
}

service MyService {
    functions {
        /// #[payable]
        Deposit() -> ();

        /// #[returns_value]
        Withdraw() -> u128;

        /// #[payable]
        /// #[returns_value]
        SwapAndRefund() -> u128;

        @query
        RegularCall() -> bool;
    }
}
"#;

#[test]

fn test_generate_payable_contract() {
    let contract = generate_solidity_contract(PAYABLE_IDL, "PayableContract");

    assert!(
        contract.is_ok(),
        "Failed to generate contract: {:?}",
        contract.err()
    );
    assert_snapshot!(String::from_utf8(contract.unwrap().data).unwrap());
}
