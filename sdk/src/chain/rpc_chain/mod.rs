use crate::types::{cell::Cell, script::Script};
use std::prelude::v1::*;

use super::{Chain, ChainError};

use ckb_jsonrpc_types::{ScriptHashType, TransactionWithStatus};
#[allow(unused_imports)]
use ckb_sdk::IndexerRpcClient;
pub use provider::*;
pub use trait_impls::*;
pub mod provider;
pub mod trait_impls;
pub mod tx_builder;
pub use tx_builder::*;

use ckb_types::{core::BlockView, packed::OutPoint, H256};

#[allow(dead_code)]
struct DefaultLock {
    out_point: OutPoint,
    code_hash: H256,
    hash_type: ScriptHashType,
}

/// Chain interface for a pair of CKB node and indexer
#[derive(Clone)]
pub struct RpcChain {
    ckb_url: String,
    indexer_url: String,
    default_lock: Option<OutPoint>,
}

impl RpcChain {
    /// Create a new RPC chain from a pair of CKB node and indexer
    ///
    /// Default lock will be set to the lock
    /// defined by the genesis block
    pub fn new(ckb_url: &str, indexer_url: &str) -> Self {
        let mut temp = Self {
            ckb_url: ckb_url.into(),
            indexer_url: indexer_url.into(),
            default_lock: None,
        };

        temp.set_sighash_all_as_default_lock();

        temp
    }

    fn set_sighash_all_as_default_lock(&mut self) {
        // Try setting SigHashAll lock script as default
        let consensus = self.inner().get_consensus();
        let tx_hash = consensus.genesis_block().tx_hashes()[0].clone();
        let lock_outpoint = OutPoint::new(tx_hash, 1); // Default location for sighashall lock cell
        self.default_lock = Some(lock_outpoint);
    }

    /// Get the last block's header
    pub fn get_tip(&self) -> Option<ckb_jsonrpc_types::HeaderView> {
        self.inner().get_tip()
    }

    /// Get a transaction from the chain
    ///
    /// Transaction status may vary, it may be pending, proposed, commited or rejected
    pub fn get_tx(&self, hash: H256) -> Result<Option<TransactionWithStatus>, ChainError> {
        self.inner().get_tx(hash)
    }

    /// Get the genesis block from the chain
    pub fn genesis_block(&self) -> Result<BlockView, ChainError> {
        self.inner().genesis_block()
    }

    /// Get the default lock as an [`OutPoint`]
    pub fn default_lock(&self) -> Option<OutPoint> {
        self.default_lock.clone()
    }

    /// Reset the chain to its genesis block. Only available for devchains.
    pub fn reset(&self) -> Result<(), ChainError> {
        self.inner().rollback(0)
    }

    /// Mine a block with all pending transactions. Only available for devchains.
    pub fn mine_once(&self) -> Result<H256, ChainError> {
        self.inner().mine_once()
    }
}
