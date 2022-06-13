// extern crate trampoline_utils;

use ckb_types::{H256, prelude::{Unpack, Pack}};
use trampoline_sdk::chain::{Chain, RpcChain};
use trampoline_utils::{account::Account, transaction::TransactionHelper};

// This tests assumes local node and indexer are running on URLs below
const CKB_URL: &str = "http://localhost:8114";
const INDEXER_URL: &str = "http://localhost:8116";

fn default_chain() -> RpcChain {
    RpcChain::new(CKB_URL)
}

// Creates default Account
// Password: trampoline
// Privkey controls default devchain account
fn default_account() -> Account {
    let password = b"trampoline";
    let pk = "d00c06bfd800d27397002dca6fb0993d5ba6399b4238b2f29ee9deb97593d2bc";
    Account::from_secret(pk.into(), password).expect("Failed creating default test Account")
}

struct DevAccount {
    pub account: Account,
    pub password: Vec<u8>,
}

impl Default for DevAccount {
    fn default() -> Self {
        let account = default_account();
        Self { account: account, password: b"trampoline".to_vec() }
    }
}

#[test]
fn test_rpc_client_get_tip() {
    let chain = default_chain();
    let tip = chain.get_tip();
    assert!(tip.is_some());
}

#[test]
fn test_verify_valid_tx() {
    // Given a default chain, default sender and random receiver account
    let chain = default_chain();
    let dev = DevAccount::default();
    let sender = dev.account;
    let password = b"trampoline";
    let destination = Account::new(b"trampoline").expect("Failed to generate random account");

    // When deploying a capacity transfer transaction
    let amount = 100_000_u64;
    let tx_helper = TransactionHelper::new(CKB_URL, INDEXER_URL);
    let tx = tx_helper.capacity_transfer(
        &sender,
        password,
        amount,
        &destination).expect("Failed to create capacity transfer transaction");

    let verification = chain.verify_tx(tx);
    assert!(verification.is_ok());
}

#[test]
fn test_send_tx_get_tx() {
    // Given a default chain, default sender and random receiver account
    let chain = default_chain();
    let dev = DevAccount::default();
    let sender = dev.account;
    let password = b"trampoline";
    let destination = Account::new(b"trampoline").expect("Failed to generate random account");

    // When deploying a capacity transfer transaction
    // WHY TF THIS AMOUNT
    // Check if change is required
    let amount = 6100000000u64;
    let tx_helper = TransactionHelper::new(CKB_URL, INDEXER_URL);
    let tx = tx_helper.capacity_transfer(
        &sender,
        password,
        amount,
        &destination).expect("Failed to create capacity transfer transaction");

    let sent_tx = tx.clone();
    
    let hash = chain.send_tx(tx);
    println!("Result from send TX is {:?}", &hash);
    let hash = hash.expect("Failed sending TX");
    let hash = H256::from(hash.unpack());

    // And retrieving the deployed transaction from chain
    let tx_in_chain = chain.get_tx(hash).expect("Failed to get TX from chain");
    let tx_in_chain  = tx_in_chain.expect("TX not found");
    let tx_in_chain = tx_in_chain.transaction.expect("TransactionWithStatus has no inner TX");
    // Then both transactions should be the same
    // let tx_in_chain = tx_in_chain.inner;


    // Transform both TXs into Trampoline::Transaction

    let sent_tx = trampoline_sdk::types::transaction::Transaction::from(sent_tx);
    let fetched_tx = trampoline_sdk::types::transaction::Transaction::from(tx_in_chain);

    assert_eq!(sent_tx, fetched_tx);
}

#[test]
fn test_sending_a_valid_transaction_returns_a_hash() {
    // Given

    // When

    // Then
}
