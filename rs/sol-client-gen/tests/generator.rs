use sails_sol_client_gen::ClientGenerator;
use std::{fs, path::PathBuf};

#[test]
fn generates_client_from_single_interface() {
    const SOL: &str = r#"
        interface IERC20 {
            function transfer(address to, uint256 amount) external returns (bool);
            function balanceOf(address owner) external view returns (uint256);
        }
    "#;

    insta::assert_snapshot!(
        ClientGenerator::from_sol(SOL)
            .with_contract_name("IERC20")
            .generate()
            .expect("generate Solidity client")
    );
}

#[test]
fn fails_on_overloaded_function() {
    const SOL: &str = r#"
        interface IOverload {
            function foo(uint256 x) external;
            function foo(address x) external;
        }
    "#;

    let err = ClientGenerator::from_sol(SOL)
        .with_contract_name("IOverload")
        .generate()
        .expect_err("overloads should fail for now");

    assert!(err.to_string().contains("overloading"));
}

#[test]
fn generates_client_from_snapshot_interface_contract() {
    let sol = solidity_from_snapshot("generator__generate_simple_contract.snap");

    insta::assert_snapshot!(
        ClientGenerator::from_sol(&sol)
            .with_contract_name("ITestContract")
            .generate()
            .expect("generate client for interface from snapshot")
    );
}

#[test]
fn generates_constructor_helpers_from_snapshot_contract() {
    let sol = solidity_from_snapshot("generator__generate_simple_contract.snap");

    insta::assert_snapshot!(
        ClientGenerator::from_sol(&sol)
            .with_contract_name("TestContractCaller")
            .generate()
            .expect("generate client for contract with constructor")
    );
}

#[test]
fn generates_constructor_helpers_from_payable_snapshot_contract() {
    let sol = solidity_from_snapshot("generator__generate_payable_contract.snap");

    insta::assert_snapshot!(
        ClientGenerator::from_sol(&sol)
            .with_contract_name("PayableContractCaller")
            .generate()
            .expect("generate client for payable contract with constructor")
    );
}

fn solidity_from_snapshot(file_name: &str) -> String {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("../sol-gen/tests/snapshots");
    path.push(file_name);

    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read snapshot {}: {e}", path.display()));
    strip_frontmatter(&raw)
}

fn strip_frontmatter(input: &str) -> String {
    let normalized = input.replace("\r\n", "\n");
    if let Some(rest) = normalized.strip_prefix("---\n")
        && let Some(end) = rest.find("\n---\n")
    {
        return rest[end + "\n---\n".len()..].to_string();
    }
    normalized
}
