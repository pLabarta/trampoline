use ckb_sdk::{unlock::UnlockError, RpcError};
use ckb_types::{core::error::OutPointError, packed::OutPoint, H256};
use ckb_verification::TransactionError;
use thiserror::Error;

use crate::types::cell::Cell;

use super::CellInputs;
#[derive(Debug, Error)]
pub enum ChainError {
    #[error(transparent)]
    TransactionError(#[from] TransactionError),
    #[error("Cannot verify transaction")]
    TransactionVerificationError,
    #[error("Failed to send transaction to network")]
    TransactionSendError,
    #[error("Failed to connect to node via RPC")]
    RpcError(RpcError),
    #[error("Failed to get Transaction from RPC")]
    GetTransactionError(H256),
    #[error("Transaction not included in any block yet")]
    TransactionNotIncluded(H256),
    #[error("Transaction not included in any block yet")]
    BlockNotFound(H256),
    #[error("Failed resolving transaction due to outpoint error")]
    TxResolveError(OutPointError),
    #[error("Genesis block not found in chain, check your chain setup")]
    GenesisBlockNotFound,
    #[error("Failed to connect to node via RPC")]
    RpcConnectionError,
    #[error("Selected lockscript not found in chain, make sure it is deployed")]
    LockScriptCellNotFound(Cell),
    #[error("Selected lockscript not found in chain, make sure it is deployed")]
    CellNotFound(OutPoint),
    #[error("Failed to unlock transaction")]
    TransactionUnlockError(UnlockError),
    #[error("Failed to unlock deploy cell transaction, it has extra lock groups")]
    DeployCellTxHasLockedGroups,
    #[error("Failed to deploy transaction due to invalid cell inputs")]
    InvalidInputs(CellInputs),
}

impl From<RpcError> for ChainError {
    fn from(e: RpcError) -> Self {
        Self::RpcError(e)
    }
}

// Most of this is taken from https://github.com/nervosnetwork/ckb-tool.
// Reimplementation here due to slight changes in the API & version conflicts
pub type ChainResult<T> = Result<T, ChainError>;
