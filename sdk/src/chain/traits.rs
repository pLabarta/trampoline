use crate::contract::{
    generator::TransactionProvider,
    schema::{BytesConversion, JsonByteConversion, MolConversion},
    Contract,
};
use crate::types::{bytes::Bytes, cell::Cell};
use ckb_types::{
    core::TransactionView,
    packed::{Byte32, OutPoint},
};

use super::{ChainError, ChainResult};
use ckb_jsonrpc_types::TransactionView as JsonTransaction;
// Modify trait TransactionProvider to be more flexible about input type
// Then define TransactionProviderError to use in Chain trait
// This way, anything that accepts TransactionProvider trait can accept chain trait
pub trait Chain {
    type Inner: TransactionProvider;

    fn inner(&self) -> Self::Inner;
    fn verify_tx<T: Into<TransactionView> + Clone>(&self, tx: T) -> ChainResult<T> {
        let view_tx: TransactionView = tx.clone().into();
        let json_tx = JsonTransaction::from(view_tx);
        if self.inner().verify_tx(json_tx) {
            Ok(tx)
        } else {
            Err(ChainError::TransactionVerificationError)
        }
    }

    fn send_tx<T: Into<TransactionView> + Clone>(&self, tx: T) -> ChainResult<Byte32> {
        let view_tx: TransactionView = tx.into();
        let json_tx = JsonTransaction::from(view_tx);
        match self.inner().send_tx(json_tx) {
            Some(hash) => Ok(hash.into()),
            None => Err(ChainError::TransactionSendError),
        }
    }

    fn deploy_cell(&mut self, cell: &Cell) -> ChainResult<OutPoint>;
    fn deploy_cells(&mut self, cells: &Vec<Cell>) -> ChainResult<Vec<OutPoint>>;

    // Removed due to changes in ckb-sdk-rust crate
    // fn genesis_info(&self) -> Option<GenesisInfo>;
    // fn set_genesis_info(&mut self, genesis_info: GenesisInfo);

    fn set_default_lock<A, D>(&mut self, lock: Contract<A, D>)
    where
        D: JsonByteConversion + MolConversion + BytesConversion + Clone + Default,
        A: JsonByteConversion + MolConversion + BytesConversion + Clone + Default;
    fn generate_cell_with_default_lock(&self, lock_args: Bytes) -> Cell;
}
