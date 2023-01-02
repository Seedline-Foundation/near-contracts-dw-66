// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import './ChainlinkConversionPath.sol';
import './interfaces/ERC20FeeProxy.sol';
import '@openzeppelin/contracts/access/Ownable.sol';

/**
 * @title Conversion
 * @notice This contract convert from chainlink then swaps ERC20 tokens
 *         before paying a request thanks to a conversion payment proxy
 */

use ink_lang as ink;

use near_sdk::{
    env,
    collections::{Map, Set},
    AccountId,
    Balance,
};

#[ink(storage)]
struct ConversionProxy {
    oracle_account: AccountId,
    provider_account: AccountId,
    owner: AccountId,
    balances: Map<AccountId, Balance>,
}

#[ink(event)]
struct TransferWithReferenceEvent {
    amount: Balance,
    currency: String,
    token_address: String,
    fee_address: AccountId,
    fee_amount: Balance,
    max_rate_timespan: u64,
    payment_reference: String,
    to: AccountId,
    crypto_amount: Balance,
    crypto_fee_amount: Balance,
}

#[ink(event)]
struct TransferErrorEvent {
    error_message: String,
}

#[ink(contract)]
impl ConversionProxy {
    fn new() -> Self {
        Self {
            oracle_account: env::predecessor_account_id(),
            provider_account: env::predecessor_account_id(),
            owner: env::predecessor_account_id(),
            balances: Map::new(),
        }
    }

    fn set_oracle_account(&mut self, account_id: AccountId) {
        self.oracle_account = account_id;
    }

    fn get_oracle_account(&self) -> AccountId {
        self.oracle_account
    }

    fn set_provider_account(&mut self, account_id: AccountId) {
        self.provider_account = account_id;
    }

    fn get_provider_account(&self) -> AccountId {
        self.provider_account
    }

    fn set_owner(&mut self, account_id: AccountId) {
        self.owner = account_id;
    }

    fn transfer_with_reference(
        &mut self,
        amount: Balance,
        currency: String,
        token_address: String,
        fee_address: AccountId,
        fee_amount: Balance,
        max_rate_timespan: u64,
        payment_reference: String,
        to: AccountId,
    ) {
        // Check if the oracle's conversion rate is within the max_rate_timespan limit
        if max_rate_timespan > 0 {
            let current_timestamp = env::block_timestamp();
            let rate_timestamp = self.get_conversion_rate_timestamp(currency, token_address);
            if current_timestamp - rate_timestamp > max_rate_timespan {
                self.emit_transfer_error("Conversion rate is too old");
                return;
            }
        }
        // Calculate the number of tokens to be transferred based on the conversion rate
        let conversion_rate = self.get_conversion_rate(currency, token_address);
        let crypto_amount = (amount * conversion_rate) / 1_000_000_000;
        let crypto_fee_amount = (fee_amount * conversion_rate) /
