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
            @indexed
            f1: u32,
            f2: String,
            @indexed
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
        @payable
        Create();
    }
    services {
        MyService
    }
}

service MyService {
    functions {
        @payable
        Deposit() -> ();

        @returns_value
        Withdraw() -> u128;

        @payable
        @returns_value
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

#[test]
fn codec_selection() {
    let idl = r#"
program CodecProgram {
    services {
        CodecTest
    }
}

service CodecTest {
    functions {
        /// Both codecs
        @entry-id: 0
        BothMethod(p1: u32) -> string;
        /// SCALE only - should be excluded from Solidity
        @entry-id: 1
        @codec: scale
        ScaleOnly(p1: u32) -> u32;
        /// Ethabi only
        @entry-id: 2
        @codec: ethabi
        EthabiOnly(p1: u32) -> u32;
        /// Payable ethabi
        @entry-id: 3
        @codec: ethabi
        @payable
        PayableEthabi(p1: u32) -> u32;
    }
}
"#;

    let contract =
        generate_solidity_contract(idl, "CodecTest").expect("generate solidity contract");
    let generated = String::from_utf8(contract.data).expect("utf8 contract");

    assert!(
        generated.contains("function codecTestBothMethod"),
        "expected codecTestBothMethod to be present"
    );
    assert!(
        generated.contains("function codecTestEthabiOnly"),
        "expected codecTestEthabiOnly to be present"
    );
    assert!(
        generated.contains("function codecTestPayableEthabi"),
        "expected codecTestPayableEthabi to be present"
    );
    assert!(
        !generated.contains("function codecTestScaleOnly"),
        "expected codecTestScaleOnly to be filtered out, got:\n{generated}"
    );

    assert_snapshot!(generated);
}
