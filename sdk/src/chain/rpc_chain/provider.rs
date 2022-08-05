use std::cell::RefCell;
use std::collections::{hash_map::Entry, HashMap, HashSet};
use std::prelude::v1::*;
use std::sync::Arc;

use ckb_chain_spec::consensus::{Consensus, ConsensusBuilder};
use ckb_hash::blake2b_256;
use ckb_jsonrpc_types as json_types;
use ckb_script::{TransactionScriptsVerifier, TxVerifyEnv};
use ckb_sdk::traits::{TransactionDependencyError, TransactionDependencyProvider};
use ckb_sdk::CkbRpcClient;
use ckb_traits::{CellDataProvider, HeaderProvider};
use ckb_types::core::cell::{
    resolve_transaction, CellMeta, CellMetaBuilder, CellProvider, CellStatus, HeaderChecker,
};
use ckb_types::core::error::OutPointError;
use ckb_types::core::{cell::ResolvedTransaction, hardfork::HardForkSwitch, TransactionView};
use ckb_types::core::{EpochNumberWithFraction, HeaderView, TransactionInfo};
use ckb_types::packed::{Byte32, CellOutput, OutPoint, Transaction as CkbTransaction};
use ckb_types::prelude::{Pack, Unpack};
use ckb_types::H256;
use ckb_util::Mutex;
use ckb_verification::NonContextualTransactionVerifier;
use json_types::TransactionWithStatus;
use lru::LruCache;

use ckb_types::bytes::Bytes;

use crate::types::transaction::Transaction;

use super::RpcChain;
use crate::chain::{ChainError, TransactionResolver, TransactionResolverError};
use crate::contract::generator::TransactionProvider;

const MAX_CYCLES: u64 = 500_0000;

/// Main data provider type for RpcChain
pub struct RpcProvider {
    chain: RefCell<RpcChain>,
    inner: Arc<Mutex<RpcProviderInner>>,
}

impl Clone for RpcProvider {
    fn clone(&self) -> RpcProvider {
        let inner = Arc::clone(&self.inner);
        let chain = self.chain.clone();
        RpcProvider { chain, inner }
    }
}

struct RpcProviderInner {
    rpc_client: CkbRpcClient,
    // indexer: IndexerRpcClient,
    tx_cache: LruCache<Byte32, TransactionView>,
    cell_cache: LruCache<OutPoint, (CellOutput, Bytes)>,
    header_cache: LruCache<Byte32, HeaderView>,
}

impl RpcProvider {
    /// Creates a new RpcProvider from a RpcChain
    pub fn new(chain: RpcChain) -> Self {
        let ref_chain = RefCell::new(chain);
        let inner = RpcProviderInner {
            rpc_client: CkbRpcClient::new(&ref_chain.borrow().ckb_url),
            // indexer: IndexerRpcClient::new(&chain.indexer_url.as_str()),
            tx_cache: LruCache::new(20),
            cell_cache: LruCache::new(20),
            header_cache: LruCache::new(20),
        };
        Self {
            chain: ref_chain,
            inner: Arc::new(Mutex::new(inner)),
        }
    }

    /// Get a transaction by its hash, return it with its status
    ///
    /// Transaction status may vary, it may be pending, proposed, commited or rejected
    pub fn get_tx(&self, hash: H256) -> Result<Option<TransactionWithStatus>, ChainError> {
        let mut inner = self.inner.lock();
        let tx = inner.rpc_client.get_transaction(hash);
        match tx {
            Ok(tx) => Ok(tx),
            Err(e) => Err(ChainError::RpcError(e)),
        }
    }

    /// Truncate the Chain to a previous block height. Only available for devchains.
    pub fn rollback(&self, previous_block: u64) -> Result<(), ChainError> {
        let mut inner = self.inner.lock();
        let block_hash = inner.rpc_client.get_header_by_number(previous_block.into());
        match block_hash {
            Ok(Some(header)) => match inner.rpc_client.truncate(header.hash) {
                Ok(()) => Ok(()),
                Err(e) => Err(ChainError::RpcError(e)),
            },

            _ => Err(ChainError::BlockNumberNotFound(previous_block)),
        }
    }

    /// Gell a pair of CellOutput and Bytes from the chain
    pub fn get_cell_with_data(
        &self,
        out_point: &OutPoint,
    ) -> Result<(CellOutput, Bytes), TransactionDependencyError> {
        let mut inner = self.inner.lock();
        if let Some(pair) = inner.cell_cache.get(out_point) {
            return Ok(pair.clone());
        }
        // TODO: handle proposed/pending transactions
        let cell_with_status = inner
            .rpc_client
            .get_live_cell(out_point.clone().into(), true)
            .map_err(|err| TransactionDependencyError::Other(err.into()))?;
        if cell_with_status.status != "live" {
            return Err(TransactionDependencyError::Other(
                format!("invalid cell status: {:?}", cell_with_status.status).into(),
            ));
        }
        let cell = cell_with_status.cell.unwrap();
        let output = CellOutput::from(cell.output);
        let output_data = cell.data.unwrap().content.into_bytes();
        inner
            .cell_cache
            .put(out_point.clone(), (output.clone(), output_data.clone()));
        Ok((output, output_data))
    }

    /// Get the last block's header
    pub fn get_tip(&self) -> Option<json_types::HeaderView> {
        let mut inner = self.inner.lock();
        let tip = inner.rpc_client.get_tip_header();
        Some(tip.unwrap())
    }

    /// Get the chain's genesis block
    pub fn genesis_block(&self) -> Result<ckb_types::core::BlockView, ChainError> {
        let mut inner = self.inner.lock();
        match inner.rpc_client.get_block_by_number(0.into()) {
            Ok(Some(block)) => Ok(block.into()),
            Ok(None) => Err(ChainError::GenesisBlockNotFound),
            Err(e) => Err(ChainError::RpcError(e)),
        }
    }

    /// Mine a block with all pending transactions. Only available for devchains.
    pub fn mine_once(&self) -> Result<H256, ChainError> {
        let mut inner = self.inner.lock();
        match inner.rpc_client.generate_block(None, None) {
            Ok(hash) => Ok(hash),
            Err(e) => Err(ChainError::RpcError(e)),
        }
    }
}

impl TransactionDependencyProvider for RpcProvider {
    fn get_transaction(
        &self,
        tx_hash: &Byte32,
    ) -> Result<TransactionView, ckb_sdk::traits::TransactionDependencyError> {
        let mut inner = self.inner.lock();
        if let Some(tx) = inner.tx_cache.get(tx_hash) {
            return Ok(tx.clone());
        }
        // TODO: handle proposed/pending transactions
        let tx_with_status = inner
            .rpc_client
            .get_transaction(tx_hash.unpack())
            .map_err(|err| TransactionDependencyError::Other(err.into()))?
            .ok_or_else(|| TransactionDependencyError::NotFound("transaction".to_string()))?;
        if tx_with_status.tx_status.status != json_types::Status::Committed {
            return Err(TransactionDependencyError::Other(
                format!("invalid transaction status: {:?}", tx_with_status.tx_status).into(),
            ));
        }
        let tx = CkbTransaction::from(tx_with_status.transaction.unwrap().inner).into_view();
        inner.tx_cache.put(tx_hash.clone(), tx.clone());
        Ok(tx)
    }

    fn get_cell(&self, out_point: &OutPoint) -> Result<CellOutput, TransactionDependencyError> {
        self.get_cell_with_data(out_point).map(|(output, _)| output)
    }
    fn get_cell_data(&self, out_point: &OutPoint) -> Result<Bytes, TransactionDependencyError> {
        self.get_cell_with_data(out_point)
            .map(|(_, output_data)| output_data)
    }
    fn get_header(&self, block_hash: &Byte32) -> Result<HeaderView, TransactionDependencyError> {
        let mut inner = self.inner.lock();
        if let Some(header) = inner.header_cache.get(block_hash) {
            return Ok(header.clone());
        }
        let header = inner
            .rpc_client
            .get_header(block_hash.unpack())
            .map_err(|err| TransactionDependencyError::Other(err.into()))?
            .map(HeaderView::from)
            .ok_or_else(|| TransactionDependencyError::NotFound("header".to_string()))?;
        inner.header_cache.put(block_hash.clone(), header.clone());
        Ok(header)
    }
}

impl HeaderProvider for RpcProvider {
    fn get_header(&self, hash: &Byte32) -> Option<HeaderView> {
        TransactionDependencyProvider::get_header(self, hash).ok()
    }
}

impl CellProvider for RpcProvider {
    fn cell(&self, out_point: &OutPoint, _eager_load: bool) -> ckb_types::core::cell::CellStatus {
        match self.get_transaction(&out_point.tx_hash()) {
            Ok(tx) => tx
                .outputs()
                .get(out_point.index().unpack())
                .map(|cell| {
                    let data = tx
                        .outputs_data()
                        .get(out_point.index().unpack())
                        .expect("output data");

                    let cell_meta = CellMetaBuilder::from_cell_output(cell, data.unpack())
                        .out_point(out_point.to_owned())
                        .build();

                    CellStatus::live_cell(cell_meta)
                })
                .unwrap_or(CellStatus::Unknown),
            Err(_err) => CellStatus::Unknown,
        }
    }
}

impl HeaderChecker for RpcProvider {
    fn check_valid(&self, block_hash: &Byte32) -> Result<(), OutPointError> {
        TransactionDependencyProvider::get_header(self, block_hash)
            .map(|_| ())
            .map_err(|_| OutPointError::InvalidHeader(block_hash.clone()))
    }
}

impl TransactionProvider for RpcProvider {
    fn send_tx(&self, tx: ckb_jsonrpc_types::TransactionView) -> Option<ckb_jsonrpc_types::Byte32> {
        let mut inner = self.inner.lock();
        let hash = inner.rpc_client.send_transaction(tx.inner, None);
        match hash {
            Ok(hash) => Some(hash.pack().into()),
            Err(e) => {
                println!("{:?}", e);
                None
            }
        }
    }

    fn verify_tx(&self, tx: ckb_jsonrpc_types::TransactionView) -> bool {
        let consensus = { self.get_consensus() };
        println!(
            "Verification Consensus: {:?}",
            consensus.secp256k1_blake160_sighash_all_type_hash
        );
        let transaction = Transaction::from(tx.inner.clone());

        println!("Starting to resolve TX");
        let resolved_tx = self.resolve_tx(transaction);
        if resolved_tx.is_err() {
            return false;
        }
        println!("Resolved TX: Ok");

        let packed_tx = CkbTransaction::from(tx.inner);
        let rtx = resolved_tx.unwrap();
        let _converted_tx_view = packed_tx.as_advanced_builder().build();
        let non_contextual = NonContextualTransactionVerifier::new(&rtx.transaction, &consensus);
        let transaction_verifier = TransactionScriptsVerifier::new(&rtx, self);
        {
            let script_verif = transaction_verifier.verify(MAX_CYCLES);
            let non_context_verif = non_contextual.verify();
            match script_verif {
                Ok(_) => println!("OK: Script verification passed"),
                Err(e) => {
                    println!("ERR: Script verification error: {:?}", e);
                    println!("Cell deps len \n{:?}", &rtx.resolved_cell_deps.len());
                    println!(
                        "Cell deps data \n{:?}",
                        &rtx.resolved_cell_deps[0].mem_cell_data_hash
                    );
                    println!(
                        "Conflicting script {:?}",
                        &rtx.resolved_inputs[0].cell_output.lock().code_hash()
                    );
                }
            }

            match non_context_verif {
                Ok(_) => println!("OK: Non-contextual verification passed"),
                Err(_e) => println!("ERR: Non-contextual erification error"),
            }
        };

        transaction_verifier.verify(MAX_CYCLES).is_ok() && non_contextual.verify().is_ok()
    }
}

impl CellDataProvider for RpcProvider {
    fn get_cell_data(&self, out_point: &OutPoint) -> Option<Bytes> {
        <RpcProvider as TransactionDependencyProvider>::get_cell_data(self, out_point).ok()
    }

    fn get_cell_data_hash(&self, out_point: &OutPoint) -> Option<Byte32> {
        <RpcProvider as TransactionDependencyProvider>::get_cell_data(self, out_point)
            .ok()
            .map(|data| blake2b_256(data.as_ref()).pack())
    }
}

impl RpcProvider {
    /// Creates a ResolvedTransaction by dereferencing its outpoints
    pub fn resolve_tx(&self, tx: Transaction) -> Result<ResolvedTransaction, ChainError> {
        let transaction = tx.into();
        let mut seen_inputs: HashSet<ckb_types::packed::OutPoint, _> = HashSet::new();

        let rtx = resolve_transaction(transaction, &mut seen_inputs, self, self);

        match rtx {
            Ok(rtx) => Ok(rtx),
            Err(e) => Err(ChainError::TxResolveError(e)),
        }
    }
}

impl RpcProvider {
    /// Get Consensus object from chain
    pub fn get_consensus(&self) -> Consensus {
        let mut inner = self.inner.lock();
        let genesis_block = {
            let response = inner
                .rpc_client
                .get_block_by_number(0.into())
                .expect("Failed to connect to RPC");
            match response {
                Some(block) => block,
                None => panic!("Genesis Block from chain is None"),
            }
        };

        let hardfork_switch = HardForkSwitch::new_builder()
            .rfc_0028(0)
            .rfc_0029(0)
            .rfc_0030(0)
            .rfc_0031(0)
            .rfc_0032(0)
            .rfc_0036(0)
            .rfc_0038(0)
            .build()
            .unwrap();

        let builder = ConsensusBuilder::default()
            .hardfork_switch(hardfork_switch)
            .genesis_block(genesis_block.into());

        builder.build()
    }
}
