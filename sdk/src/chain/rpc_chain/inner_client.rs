use ckb_chain_spec::consensus::Consensus;
use ckb_sdk::{CkbRpcClient, IndexerRpcClient};
use ckb_types::packed::{Bytes, CellOutput, OutPoint};
use lru::LruCache;

pub struct RpcClient {
    pub ckb: CkbRpcClient,
    pub indexer: IndexerRpcClient,
}

impl RpcClient {
    pub fn new(ckb_url: &str, indexer_url: &str, cap: usize) -> Self {
        Self {
            ckb: CkbRpcClient::new(ckb_url),
            indexer: IndexerRpcClient::new(indexer_url),
        }
    }
}
