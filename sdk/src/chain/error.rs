use ckb_verification::TransactionError;
use thiserror::Error;
#[derive(Debug, Error)]
pub enum ChainError {
    #[error(transparent)]
    TransactionError(#[from] TransactionError),
    #[error("Cannot verify transaction")]
    TransactionVerificationError,
    #[error("Failed to send transaction to network")]
    TransactionSendError,
    #[error("Failed to connect to node via RPC")]
    RpcError,
}

// Most of this is taken from https://github.com/nervosnetwork/ckb-tool.
// Reimplementation here due to slight changes in the API & version conflicts
pub type ChainResult<T> = Result<T, ChainError>;
