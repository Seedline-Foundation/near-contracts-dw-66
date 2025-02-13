use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::Timestamp;
use near_sdk::{env, near_bindgen, AccountId};

near_sdk::setup_alloc!();

/**
 * Mocking the Flux oracle contract
 */

// Return type the Flux price oracle
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct PriceEntry {
  pub price: U128,            // Last reported price
  pub decimals: u16,          // Amount of decimals (e.g. if 2, 100 = 1.00)
  pub last_update: Timestamp, // Time of report
}

// For mocks: state of Flux price oracle
#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct FPOContract {}

/**
 * Mocked FPO contract for tests
 */

#[near_bindgen]
impl FPOContract {
  #[allow(unused_variables)]
  pub fn get_entry(&self, pair: String, provider: AccountId) -> Option<PriceEntry> {
    env::log(format!("get_entry OK").as_bytes());
    match &*pair {
      "NEAR/USD" => Some(PriceEntry {
        // 1 USD = 123 NEAR, 10 nanoseconds ago
        price: U128::from(123000000),
        decimals: 6,
        last_update: env::block_timestamp() - 10,
      }),
      _ => None,
    }
  }
}

#[cfg(test)]
mod tests {

  use super::*;
  use crate::AccountId;
  use near_sdk::{testing_env, Balance, Gas, MockedBlockchain, VMContext};
  use near_sdk_sim::to_yocto;

  fn get_context(
    predecessor_account_id: AccountId,
    attached_deposit: Balance,
    prepaid_gas: Gas,
    is_view: bool,
  ) -> VMContext {
    VMContext {
      current_account_id: predecessor_account_id.clone(),
      signer_account_id: predecessor_account_id.clone(),
      signer_account_pk: vec![0, 1, 2],
      predecessor_account_id,
      input: vec![],
      block_index: 1,
      block_timestamp: 0,
      epoch_height: 1,
      account_balance: 0,
      account_locked_balance: 0,
      storage_usage: 10u64.pow(6),
      attached_deposit,
      prepaid_gas,
      random_seed: vec![0, 1, 2],
      is_view,
      output_data_receivers: vec![],
    }
  }
  #[test]
  fn get_entry() {
    let context = get_context("alice.near".to_string(), to_yocto("1"), 10u64.pow(14), true);
    testing_env!(context);
    let contract = FPOContract::default();
    if let Some(result) = contract.get_entry("NEAR/USD".to_string(), "any".to_string()) {
      assert_eq!(result.price, U128::from(123000000));
      assert_eq!(result.decimals, 6);
    } else {
      panic!("NEAR/USD mock returned None")
    }
  }
  #[test]
  fn get_missing_pair_entry() {
    let context = get_context("alice.near".to_string(), to_yocto("1"), 10u64.pow(14), true);
    testing_env!(context);
    let contract = FPOContract::default();
    assert_eq!(
      contract
        .get_entry("NEAR/WRONG".to_string(), "any".to_string())
        .is_none(),
      true
    );
  }
}
