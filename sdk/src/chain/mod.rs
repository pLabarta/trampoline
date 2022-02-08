use ckb_types::{
    bytes::Bytes,
    core::TransactionView,
    packed::{Byte32, CellOutput},
    prelude::*,
};
use ckb_verification::TransactionError;
pub type CellOutputWithData = (CellOutput, Bytes);
mod mock_chain;
pub use mock_chain::*;

// Most of this is taken from https://github.com/nervosnetwork/ckb-tool.
// Reimplementation here due to slight changes in the API & version conflicts

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Message {
    pub id: Byte32,
    pub message: String,
}

pub struct OutputsDataVerifier<'a> {
    transaction: &'a TransactionView,
}

impl<'a> OutputsDataVerifier<'a> {
    pub fn new(transaction: &'a TransactionView) -> Self {
        Self { transaction }
    }

    pub fn verify(&self) -> Result<(), TransactionError> {
        let outputs_len = self.transaction.outputs().len();
        let outputs_data_len = self.transaction.outputs_data().len();

        if outputs_len != outputs_data_len {
            return Err(TransactionError::OutputsDataLengthMismatch {
                outputs_data_len,
                outputs_len,
            });
        }
        Ok(())
    }
}
