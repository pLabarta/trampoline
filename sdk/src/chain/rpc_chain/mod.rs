use super::Chain;

pub use trait_impls::*;
pub mod trait_impls;
pub mod provider;


use ckb_types::{
    core::{
        cell::{HeaderChecker}
    },
    packed::{ OutPoint},
};

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
}

impl HeaderChecker for RpcChain {
    fn check_valid(
        &self,
        block_hash: &ckb_types::packed::Byte32,
    ) -> Result<(), ckb_types::core::error::OutPointError> {
        todo!()
    }
}