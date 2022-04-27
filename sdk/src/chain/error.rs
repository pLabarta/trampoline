use ckb_verification::TransactionError;
use thiserror::Error;
use ckb_sdk::ParseGenesisInfoError;
#[derive(Debug, Error)]
pub enum ChainError {
    #[error(transparent)]
    TransactionError(#[from] TransactionError),
    #[error(transparent)]
    GenesisInfoError(#[from] ParseGenesisInfoError),
    #[error("Cannot verify transaction")]
    TransactionVerificationError,
    #[error("Failed to send transaction to network")]
    TransactionSendError,
}


// Most of this is taken from https://github.com/nervosnetwork/ckb-tool.
// Reimplementation here due to slight changes in the API & version conflicts
pub type ChainResult<T> = Result<T, ChainError>;