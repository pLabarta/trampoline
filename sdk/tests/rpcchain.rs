
// extern crate trampoline_utils;

use trampoline_sdk::chain::{RpcChain};
use trampoline_utils::account::Account;

// This tests assumes local node and indexer are running on URLs below
fn default_chain() -> RpcChain {
    RpcChain::new("http://localhost:8114")
}

// Creates default Account
// Password: trampoline
// Privkey controls default devchain account
fn default_account() -> Account {
    let password = b"trampoline";
    let pk = "d00c06bfd800d27397002dca6fb0993d5ba6399b4238b2f29ee9deb97593d2bc";
    Account::from_secret(pk.into(), password).expect("Failed creating default test Account")
}

#[test]
fn test_rpc_client_get_tip() {
    let chain = default_chain();
    let tip = chain.get_tip();
    assert!(tip.is_some());
}

#[test]
fn test_send_tx() {
    // Given
    let chain = default_chain();
    let account = default_account();
    // When deploying a capacity transfer transaction
    
}

#[test]
fn test_sending_a_valid_transaction_returns_a_hash () {
    // Given

    // When

    // Then
}