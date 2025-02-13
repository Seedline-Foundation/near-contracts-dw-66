use near_sdk_sim::transaction::ExecutionStatus;
use near_sdk_sim::{lazy_static_include, to_yocto, ExecutionResult};

lazy_static_include::lazy_static_include_bytes! {
   REQUEST_PROXY_BYTES => "./out/conversion_proxy.wasm"
}

pub fn assert_almost_eq_with_max_delta(left: u128, right: u128, max_delta: u128) {
    assert!(
        std::cmp::max(left, right) - std::cmp::min(left, right) <= max_delta,
        "{}",
        format!(
            "Left {} is not even close to Right {} within delta {}",
            left, right, max_delta
        )
    );
}

pub fn assert_eq_with_gas(left: u128, right: u128) {
    assert_almost_eq_with_max_delta(left, right, to_yocto("0.005"));
}

pub fn assert_one_promise_error(promise_result: ExecutionResult, expected_error_message: &str) {
    assert_eq!(promise_result.promise_errors().len(), 1);

    if let ExecutionStatus::Failure(execution_error) = &promise_result
        .promise_errors()
        .remove(0)
        .unwrap()
        .outcome()
        .status
    {
        assert!(execution_error.to_string().contains(expected_error_message));
    } else {
        unreachable!();
    }
}
