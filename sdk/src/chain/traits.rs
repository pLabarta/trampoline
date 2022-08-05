use crate::types::{bytes::Bytes, cell::Cell};
use crate::{contract::generator::TransactionProvider, types::script::Script};
use ckb_sdk::{unlock::ScriptUnlocker, ScriptId};
use std::collections::HashMap;
use std::prelude::v1::*;

use crate::ckb_types::{
    core::TransactionView,
    packed::{Byte32, OutPoint},
};

/// Collections of scripts and their unlockers for signing transactions
pub type Unlockers = HashMap<ScriptId, Box<dyn ScriptUnlocker>>;

/// Variants for specifing cell inputs to be consumed by deploy_cell methods
#[derive(Debug, Clone)]
pub enum CellInputs {
    /// Match cells with specified script
    ScriptQuery(Script),
    /// Run a Chain's default cell collection to balance the transaction
    Empty,
}

impl From<Script> for CellInputs {
    fn from(script: Script) -> Self {
        CellInputs::ScriptQuery(script)
    }
}

use super::{ChainError, ChainResult};
use ckb_jsonrpc_types::TransactionView as JsonTransaction;

/// Universal API for interacting with CKB blockchains
///
/// This trait is implemented by [`RpcChain`] and [`MockChain`]. It provides
/// a standard interface for verifying and sending transactions, deploying cells
/// and specifying the default lock these methods should use.
pub trait Chain {
    // Modify trait TransactionProvider to be more flexible about input type
    // Then define TransactionProviderError to use in Chain trait
    // This way, anything that accepts TransactionProvider trait can accept chain trait

    /// Inner type that provides transactions verification and sending methods
    type Inner: TransactionProvider;

    /// Returns the inner Transaction Prpvider
    fn inner(&self) -> Self::Inner;

    /// Verify if a transaction is valid given the current chain state
    fn verify_tx<T: Into<TransactionView> + Clone>(&self, tx: T) -> ChainResult<T> {
        let view_tx: TransactionView = tx.clone().into();
        let json_tx = JsonTransaction::from(view_tx);
        if self.inner().verify_tx(json_tx) {
            Ok(tx)
        } else {
            Err(ChainError::TransactionVerificationError)
        }
    }

    /// Publish a transaction the the chain, returns its hash
    fn send_tx<T: Into<TransactionView> + Clone>(&self, tx: T) -> ChainResult<Byte32> {
        let view_tx: TransactionView = tx.into();
        let json_tx = JsonTransaction::from(view_tx);
        match self.inner().send_tx(json_tx) {
            Some(hash) => Ok(hash.into()),
            None => Err(ChainError::TransactionSendError),
        }
    }

    /// Deploys a Cell to the chain
    ///
    /// Cell Inputs must be provided, but if empty
    /// cells from the Unlocker address will be used.
    fn deploy_cell(
        &mut self,
        cell: &Cell,
        unlockers: Unlockers,
        inputs: &CellInputs,
    ) -> ChainResult<OutPoint>;

    /// Deploys multiples Cells to the chain
    ///
    /// Cell Inputs must be provided, but if empty
    /// cells from the Unlocker address will be used.
    fn deploy_cells(
        &mut self,
        cells: &[Cell],
        unlockers: Unlockers,
        inputs: &CellInputs,
    ) -> ChainResult<Vec<OutPoint>>;

    /// Changes the chain default lock
    fn set_default_lock(&mut self, lock: OutPoint) -> Result<(), ChainError>;

    /// Creates a default cell using the Chain's default lock
    fn generate_cell_with_default_lock(&self, lock_args: Bytes) -> Cell;
}
