use super::{Chain, ChainError};

use ckb_jsonrpc_types::TransactionWithStatus;
pub use trait_impls::*;
pub mod provider;
pub mod trait_impls;

use ckb_types::{core::cell::HeaderChecker, packed::OutPoint, H256};

#[derive(Clone)]
pub struct RpcChain {
    ckb_url: String,
}

impl RpcChain {
    pub fn new(ckb_url: &str) -> Self {
        Self {
            ckb_url: ckb_url.into(),
        }
    }

    pub fn get_tip(&self) -> Option<ckb_jsonrpc_types::HeaderView> {
        self.inner().get_tip().clone()
    }

    pub fn get_tx(&self, hash: H256) -> Result<Option<TransactionWithStatus>, ChainError> {
        self.inner().get_tx(hash)
    }
}

impl HeaderChecker for RpcChain {
    fn check_valid(
        &self,
        block_hash: &ckb_types::packed::Byte32,
    ) -> Result<(), ckb_types::core::error::OutPointError> {
        todo!()
    }
}
