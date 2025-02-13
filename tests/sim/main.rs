use crate::utils::*;
use conversion_proxy::ConversionProxyContract;
use mocks::FPOContractContract;
use near_sdk::json_types::{U128, U64};
use near_sdk_sim::init_simulator;
use near_sdk_sim::runtime::GenesisConfig;
use near_sdk_sim::ContractAccount;
use near_sdk_sim::UserAccount;
use near_sdk_sim::{call, deploy, lazy_static_include, to_yocto};
use std::convert::TryInto;

near_sdk::setup_alloc!();

const CONTRACT_ID: &str = "request_proxy";
lazy_static_include::lazy_static_include_bytes! {
   REQUEST_PROXY_BYTES => "out/conversion_proxy.wasm"
}
lazy_static_include::lazy_static_include_bytes! {
   MOCKED_FPO_BYTES => "out/mocks.wasm"
}

mod utils;

// Initialize test environment with 3 accounts (alice, bob, builder) and a conversion mock.
fn init() -> (
    UserAccount,
    UserAccount,
    UserAccount,
    ContractAccount<ConversionProxyContract>,
) {
    let genesis = GenesisConfig::default();
    let root = init_simulator(Some(genesis));

    deploy!(
        contract: FPOContractContract,
        contract_id: "mockedfpo".to_string(),
        bytes: &MOCKED_FPO_BYTES,
        signer_account: root,
        deposit: to_yocto("5")
    );

    let account = root.create_user("alice".to_string(), to_yocto("1000"));

    let zero_balance: u128 = 1820000000000000000000;
    let empty_account_1 = root.create_user("bob".parse().unwrap(), zero_balance);
    let empty_account_2 = root.create_user("builder".parse().unwrap(), zero_balance);

    let request_proxy = deploy!(
        contract: ConversionProxyContract,
        contract_id: CONTRACT_ID,
        bytes: &REQUEST_PROXY_BYTES,
        signer_account: root,
        deposit: to_yocto("5"),
        init_method: new("mockedfpo".into(), "any".into())
    );

    let get_oracle_result = call!(root, request_proxy.get_oracle_account());
    get_oracle_result.assert_success();

    debug_assert_eq!(
        &get_oracle_result.unwrap_json_value().to_owned(),
        &"mockedfpo".to_string()
    );

    (account, empty_account_1, empty_account_2, request_proxy)
}

#[test]
fn test_transfer_usd_near() {
    let (alice, bob, builder, request_proxy) = init();
    let initial_alice_balance = alice.account().unwrap().amount;
    let initial_bob_balance = bob.account().unwrap().amount;
    let initial_builder_balance = builder.account().unwrap().amount;
    let transfer_amount = to_yocto("100");
    let payment_address = bob.account_id().try_into().unwrap();
    let fee_address = builder.account_id().try_into().unwrap();

    // Token transfer failed
    let result = call!(
        alice,
        request_proxy.transfer_with_reference(
            "0x1122334455667788".to_string(),
            payment_address,
            // 12.00 USD (main)
            U128::from(1200),
            String::from("USD"),
            fee_address,
            // 1.00 USD (fee)
            U128::from(100),
            U64::from(0)
        ),
        deposit = transfer_amount
    );
    result.assert_success();

    println!(
        "test_transfer_usd_near ==> TeraGas burnt: {}",
        result.gas_burnt() as f64 / 1e12
    );

    let alice_balance = alice.account().unwrap().amount;
    assert!(alice_balance < initial_alice_balance);
    let spent_amount = initial_alice_balance - alice_balance;
    assert!(
        spent_amount - to_yocto("13") / 123 < to_yocto("0.005"),
        "Alice should spend 12 + 1 USD worth of NEAR (+ gas)",
    );
    println!(
        "diff: {}",
        (spent_amount - to_yocto("13") / 123) / 1_000_000_000_000_000_000_000_000
    );

    assert!(bob.account().unwrap().amount > initial_bob_balance);
    let received_amount = bob.account().unwrap().amount - initial_bob_balance;
    assert_eq!(
        received_amount,
        // 12 USD / rate mocked
        to_yocto("12") / 123,
        "Bob should receive exactly 12 USD worth of NEAR"
    );

    assert!(builder.account().unwrap().amount > initial_builder_balance);
    let received_amount = builder.account().unwrap().amount - initial_builder_balance;
    assert_eq!(
        received_amount,
        // 1 USD / rate mocked
        to_yocto("1") / 123,
        "Builder should receive exactly 1 USD worth of NEAR"
    );
}

#[test]
fn test_transfer_with_invalid_reference_length() {
    let transfer_amount = to_yocto("500");

    let (alice, bob, builder, request_proxy) = init();
    let payment_address = bob.account_id().try_into().unwrap();
    let fee_address = builder.account_id().try_into().unwrap();

    // Token transfer failed
    let result = call!(
        alice,
        request_proxy.transfer_with_reference(
            "0x11223344556677".to_string(),
            payment_address,
            U128::from(12),
            String::from("USD"),
            fee_address,
            U128::from(1),
            U64::from(0)
        ),
        deposit = transfer_amount
    );
    // No successful outcome is expected
    assert!(!result.is_ok());

    println!(
        "test_transfer_with_invalid_parameter_length > TeraGas burnt: {}",
        result.gas_burnt() as f64 / 1e12
    );

    assert_one_promise_error(result, "Incorrect payment reference length");

    // Check Alice balance
    assert_eq_with_gas(to_yocto("1000"), alice.account().unwrap().amount);
}

#[test]
fn test_transfer_with_wrong_currency() {
    let (alice, bob, builder, request_proxy) = init();
    let transfer_amount = to_yocto("100");
    let payment_address = bob.account_id().try_into().unwrap();
    let fee_address = builder.account_id().try_into().unwrap();

    // Token transfer failed
    let result = call!(
        alice,
        request_proxy.transfer_with_reference(
            "0x1122334455667788".to_string(),
            payment_address,
            U128::from(1200),
            String::from("WRONG"),
            fee_address,
            U128::from(100),
            U64::from(0)
        ),
        deposit = transfer_amount
    );
    assert_one_promise_error(result, "ERR_INVALID_ORACLE_RESPONSE");
}

#[test]
fn test_transfer_zero_usd_near() {
    let (alice, bob, builder, request_proxy) = init();
    let initial_alice_balance = alice.account().unwrap().amount;
    let initial_bob_balance = bob.account().unwrap().amount;
    let transfer_amount = to_yocto("100");
    let payment_address = bob.account_id().try_into().unwrap();
    let fee_address = builder.account_id().try_into().unwrap();

    let result = call!(
        alice,
        request_proxy.transfer_with_reference(
            "0x1122334455667788".to_string(),
            payment_address,
            U128::from(0),
            String::from("USD"),
            fee_address,
            U128::from(0),
            U64::from(0)
        ),
        deposit = transfer_amount
    );
    result.assert_success();

    let alice_balance = alice.account().unwrap().amount;
    assert!(alice_balance < initial_alice_balance);
    let spent_amount = initial_alice_balance - alice_balance;
    assert!(
        spent_amount < to_yocto("0.005"),
        "Alice should not spend NEAR on a 0 USD payment",
    );

    assert!(
        bob.account().unwrap().amount == initial_bob_balance,
        "Bob's balance should be unchanged"
    );
    assert!(
        builder.account().unwrap().amount == initial_bob_balance,
        "Builder's balance should be unchanged"
    );
}

#[test]
fn test_outdated_rate() {
    let (alice, bob, builder, request_proxy) = init();
    let transfer_amount = to_yocto("100");
    let payment_address = bob.account_id().try_into().unwrap();
    let fee_address = builder.account_id().try_into().unwrap();

    let result = call!(
        alice,
        request_proxy.transfer_with_reference(
            "0x1122334455667788".to_string(),
            payment_address,
            U128::from(0),
            String::from("USD"),
            fee_address,
            U128::from(0),
            // The mocked rate is 10 nanoseconds old
            U64::from(1)
        ),
        deposit = transfer_amount
    );
    assert_one_promise_error(result, "Conversion rate too old");
}
