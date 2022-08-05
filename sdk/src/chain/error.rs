use ckb_sdk::{unlock::UnlockError, RpcError};
use ckb_types::{core::error::OutPointError, packed::OutPoint, H256};
use ckb_verification::TransactionError;
use std::prelude::v1::*;
use thiserror::Error;

use crate::types::cell::Cell;

use super::CellInputs;

/// Error types for Chain methods
#[derive(Debug, Error)]
pub enum ChainError {
    /// Malformed transaction
    #[error(transparent)]
    TransactionError(#[from] TransactionError),
    /// Failed to verify transaction
    #[error("Cannot verify transaction")]
    TransactionVerificationError,
    /// Failed to publish transaction to the network
    #[error("Failed to send transaction to network")]
    TransactionSendError,
    /// Failed to connect to node via RPC
    #[error("Failed to connect to node via RPC")]
    RpcError(#[from] RpcError),
    /// Failed ton get transaction from node
    #[error("Failed to get Transaction from RPC")]
    GetTransactionError(H256),
    /// Transaction not included in any block
    #[error("Transaction with hash {0}not included in any block yet")]
    TransactionNotIncluded(H256),
    /// Block not found in chain
    #[error("Transaction with hash {0} not included in any block yet")]
    BlockNotFound(H256),
    /// Block with number has not been mined yet
    #[error("Block with number {0} was not mined yet")]
    BlockNumberNotFound(u64),
    /// Failed to resolve transaction due to outpoint error
    #[error("Failed resolving transaction due to outpoint error")]
    TxResolveError(#[from] OutPointError),
    /// Genesis block not found
    #[error("Genesis block not found in chain, check your chain setup")]
    GenesisBlockNotFound,
    /// Failed to get cell with specified outpoint
    #[error("Cell with outpoint {0} make sure it is deployed")]
    CellNotFound(OutPoint),
    /// Failed to unlock transaction
    #[error("Failed to unlock transaction")]
    TransactionUnlockError(#[from] UnlockError),
    /// Transaction cannot be deployed, still has unlocked inputs
    #[error("Failed to unlock deploy cell transaction, it has extra lock groups")]
    DeployCellTxHasLockedGroups,
    /// Transaction has invalid cell inputs
    #[error("Failed to deploy transaction due to invalid cell inputs")]
    InvalidInputs(CellInputs),
}

// Most of this is taken from https://github.com/nervosnetwork/ckb-tool.
// Reimplementation here due to slight changes in the API & version conflicts
/// Result type for Chain methods
pub type ChainResult<T> = Result<T, ChainError>;
