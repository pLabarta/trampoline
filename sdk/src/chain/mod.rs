
mod rpc_chain;
mod mock_chain;
mod traits;
mod error;
pub use mock_chain::*;
pub use traits::*;
pub use error::*; 

use crate::contract::Contract;
use crate::types::cell::{Cell, CellOutputWithData};
use ckb_types::{
   
    core::TransactionView,
    packed::Byte32,
    prelude::*,
};
use ckb_verification::TransactionError;
use ckb_sdk::GenesisInfo;


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
