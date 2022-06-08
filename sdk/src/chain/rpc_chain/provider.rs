use std::cell::RefCell;
use std::collections::HashSet;
use std::sync::Arc;

use ckb_chain_spec::consensus::{Consensus, ConsensusBuilder};
use ckb_hash::blake2b_256;
use ckb_jsonrpc_types as json_types;
use ckb_script::{TransactionScriptsVerifier, TxVerifyEnv};
use ckb_sdk::traits::{TransactionDependencyError, TransactionDependencyProvider};
use ckb_sdk::{CkbRpcClient};
use ckb_traits::{HeaderProvider, CellDataProvider};
use ckb_types::core::cell::{resolve_transaction_with_options, CellProvider, CellMetaBuilder, CellStatus, HeaderChecker, ResolveOptions};
use ckb_types::core::error::OutPointError;
use ckb_types::core::{HeaderView, EpochNumberWithFraction};
use ckb_types::core::{cell::ResolvedTransaction, hardfork::HardForkSwitch, TransactionView};
use ckb_types::packed::{Byte32, OutPoint, Transaction, CellOutput};
use ckb_types::prelude::{Pack, Unpack};
use ckb_util::Mutex;
use ckb_verification::{NonContextualTransactionVerifier};
use lru::LruCache;

use ckb_types::bytes::Bytes;

use crate::contract::generator::TransactionProvider;
use crate::types::transaction;
use super::RpcChain;

const MAX_CYCLES: u64 = 500_0000;




// RpcChain Transaction provider type
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

    pub fn resolve_tx(&self, tx: TransactionView) -> Result<ResolvedTransaction, OutPointError> {
        let mut seen_inputs = HashSet::new();
        let hardfork_switch = HardForkSwitch::new_without_any_enabled()
        .as_builder()
        .rfc_0032(200)
        .build()
        .unwrap();
        let resolve_opts = ResolveOptions::new()
            .apply_current_features(&hardfork_switch, 300);
        resolve_transaction_with_options(
            tx,
            &mut seen_inputs,
            self,
            self,
            resolve_opts,
        )
    }

    pub fn get_tip(&self) -> Option<json_types::HeaderView> {
        let mut inner = self.inner.lock();
        let tip = inner.rpc_client.get_tip_header();
        Some(tip.unwrap())
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
        let tx = Transaction::from(tx_with_status.transaction.unwrap().inner).into_view();
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
    fn cell(&self, out_point: &OutPoint, eager_load: bool) -> ckb_types::core::cell::CellStatus {
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
            _ => None,
        }
    }

    fn verify_tx(&self, tx: ckb_jsonrpc_types::TransactionView) -> bool {
        let inner = self.inner.lock();
        let consensus = dummy_consensus();
        let inner_tx = tx.inner;
        let packed_tx = ckb_types::packed::Transaction::from(inner_tx.clone());
        let core_tx = transaction::Transaction::from(inner_tx);
        let core_tx_view = TransactionView::from(core_tx);
        let resolved_tx = self.resolve_tx(core_tx_view);
        if resolved_tx.is_err() {
            return false;
        }

        let tx_env = {
            let epoch = EpochNumberWithFraction::new(300, 0, 1);
            let header = HeaderView::new_advanced_builder()
                .epoch(epoch.pack())
                .build();
            TxVerifyEnv::new_commit(&header)
        };

        let rtx = resolved_tx.unwrap();
        let converted_tx_view = packed_tx.as_advanced_builder().build();
        let non_contextual = NonContextualTransactionVerifier::new(&converted_tx_view, &consensus);
        let transaction_verifier = TransactionScriptsVerifier::new(&rtx, &consensus, self,&tx_env);
        if transaction_verifier.verify(MAX_CYCLES).is_ok() && non_contextual.verify().is_ok() {
            return true;
        } else {
            return false;
        }
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

// impl CellProvider for RpcProvider {
//     fn cell(&self, out_point: &ckb_types::packed::OutPoint, eager_load: bool) -> ckb_types::core::cell::CellStatus {
//         let chain = self.chain.borrow_mut();
//         let client = chain.client.lock();
//         let cell = client.ckb.get_live_cell(out_point.unpack().into())
//     }
// }

pub fn dummy_consensus() -> Consensus {
    let hardfork_switch = HardForkSwitch::new_without_any_enabled()
        .as_builder()
        .rfc_0032(200)
        .build()
        .unwrap();
    ConsensusBuilder::default()
        .hardfork_switch(hardfork_switch)
        .build()
}
