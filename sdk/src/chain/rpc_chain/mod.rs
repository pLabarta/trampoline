use super::Chain;
use ckb_util::Mutex;
use trait_impls::*;
pub mod trait_impls;
use inner_client::*;
pub mod inner_client;
use provider::*;
pub mod provider;

use crate::contract::generator::TransactionProvider;
use ckb_chain_spec::consensus::{
    build_genesis_epoch_ext, Consensus, ConsensusBuilder, ConsensusProvider, ProposalWindow,
};
use ckb_script::TransactionScriptsVerifier;
use ckb_sdk::{CkbRpcClient, IndexerRpcClient};
use ckb_types::{
    core::{
        cell::{resolve_transaction_with_options, HeaderChecker, ResolvedTransaction},
        hardfork::HardForkSwitch,
        Ratio, RationalU256, TransactionView,
    },
    packed::{Bytes, CellOutput, OutPoint},
    prelude::Pack,
};
use ckb_verification::NonContextualTransactionVerifier;
use std::{cell::RefCell, collections::HashSet, sync::Arc};

#[derive(Clone)]
pub struct RpcChain {
    ckb_url: String,
    indexer_url: String,
}

impl RpcChain {
    pub fn new(ckb_url: &str, indexer_url: &str) -> Self {
        Self {
            ckb_url: ckb_url.into(),
            indexer_url: indexer_url.into(),
        }
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

// fn rational_to_ratio(r: RationalU256) -> Ratio {
//     let list: Vec<&str> = r.to_string().split("/").collect();
//     let numer = list[0].parse::<u64>().unwrap();
//     let denom = list[1].parse::<u64>().unwrap();
//     Ratio::new(numer, denom)
// }
