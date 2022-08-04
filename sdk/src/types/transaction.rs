//! Types for handling and creating CKB transactions

use super::cell::CellOutputWithData;
use crate::ckb_types::core::cell::CellMeta;
use crate::ckb_types::core::{TransactionBuilder, TransactionView};
use crate::ckb_types::packed::{
    Transaction as PackedTransaction, TransactionView as PackedTransactionView,
    TransactionViewBuilder,
};
use crate::ckb_types::prelude::*;
use ckb_jsonrpc_types::{Transaction as JsonTransaction, TransactionView as JsonTransactionView};
use std::prelude::v1::*;

// core::TransactionView has Transaction, hash, and witness_hash
// ckb_jsonrpc_types::TransactionView has Transaction and hash

/// Wrapper for the JsonTransaction type
#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct Transaction {
    pub(crate) inner: JsonTransaction,
}

/// Type that combines a TX and input cells used by the generator
#[derive(Clone, Debug)]
pub struct CellMetaTransaction {
    /// Full transaction view
    pub tx: TransactionView,
    /// Cells included in the transaction
    pub inputs: Vec<CellMeta>,
}

impl From<JsonTransactionView> for Transaction {
    fn from(view: JsonTransactionView) -> Self {
        Self { inner: view.inner }
    }
}

impl From<JsonTransaction> for Transaction {
    fn from(tx: JsonTransaction) -> Self {
        Self { inner: tx }
    }
}

impl From<TransactionView> for Transaction {
    fn from(core_view: TransactionView) -> Self {
        let json_view = JsonTransactionView::from(core_view);
        Self {
            inner: json_view.inner,
        }
    }
}

impl From<PackedTransaction> for Transaction {
    fn from(packed_tx: PackedTransaction) -> Self {
        Self {
            inner: packed_tx.into(),
        }
    }
}

impl From<PackedTransactionView> for Transaction {
    fn from(packed_view: PackedTransactionView) -> Self {
        Self {
            inner: JsonTransactionView::from(packed_view.unpack()).inner,
        }
    }
}

impl From<Transaction> for JsonTransaction {
    fn from(tx: Transaction) -> Self {
        tx.inner
    }
}

impl From<Transaction> for PackedTransaction {
    fn from(tx: Transaction) -> Self {
        tx.inner.into()
    }
}

impl From<Transaction> for PackedTransactionView {
    fn from(tx: Transaction) -> Self {
        TransactionViewBuilder::default().data(tx.into()).build()
    }
}
impl From<Transaction> for JsonTransactionView {
    fn from(tx: Transaction) -> Self {
        TransactionView::from(tx).into()
    }
}
impl From<Transaction> for TransactionView {
    fn from(tx: Transaction) -> Self {
        //let packed_tx: PackedTransaction = tx.into();
        //let packed_view = PackedTransactionView::new_builder().data(packed_tx).build();
        //packed_view.unpack()
        PackedTransactionView::from(tx).unpack()
    }
}

impl From<TransactionView> for CellMetaTransaction {
    fn from(tx: TransactionView) -> Self {
        Self { tx, inputs: vec![] }
    }
}

impl From<Transaction> for CellMetaTransaction {
    fn from(trampoline_tx: Transaction) -> Self {
        Self {
            tx: trampoline_tx.into(),
            inputs: vec![],
        }
    }
}

impl From<CellMetaTransaction> for Transaction {
    fn from(cm_tx: CellMetaTransaction) -> Self {
        cm_tx.tx.into()
    }
}

impl CellMetaTransaction {
    /// Return a copy using the specified Transaction
    pub fn tx(self, tx: TransactionView) -> Self {
        Self {
            tx,
            inputs: self.inputs,
        }
    }

    /// Returns a copy using the specified inputs
    pub fn with_inputs(self, inputs: Vec<CellMeta>) -> Self {
        Self {
            tx: self.tx,
            inputs,
        }
    }

    /// Returns a TransactionBuilder from the inner TX
    pub fn as_advanced_builder(&self) -> TransactionBuilder {
        self.tx.as_advanced_builder()
    }

    /// Returns CellDeps from the inner TX
    pub fn cell_deps(&self) -> crate::ckb_types::packed::CellDepVec {
        self.tx.cell_deps()
    }

    /// Returns CellInputs from the inner TX
    pub fn inputs(&self) -> crate::ckb_types::packed::CellInputVec {
        self.tx.inputs()
    }

    /// Returns CellOutputs from the inner TX
    pub fn outputs(&self) -> crate::ckb_types::packed::CellOutputVec {
        self.tx.outputs()
    }

    /// Returns outputs data vector from the inner TX
    pub fn outputs_data(&self) -> crate::ckb_types::packed::BytesVec {
        self.tx.outputs_data()
    }

    /// Returns Witnesses from the inner TX
    pub fn witnesses(&self) -> crate::ckb_types::packed::BytesVec {
        self.tx.witnesses()
    }

    /// Returns Output with specified index from the inner TX
    pub fn output(&self, idx: usize) -> Option<crate::ckb_types::packed::CellOutput> {
        self.tx.output(idx)
    }

    /// Returns Output with data with specified index from the inner TX
    pub fn output_with_data(&self, idx: usize) -> Option<CellOutputWithData> {
        self.tx.output_with_data(idx)
    }

    /// Returns Outpoints the inner TX
    pub fn output_pts(&self) -> Vec<crate::ckb_types::packed::OutPoint> {
        self.tx.output_pts()
    }

    /// Returns CellDeps as an interator from the inner TX
    pub fn cell_deps_iter(&self) -> impl Iterator<Item = crate::ckb_types::packed::CellDep> {
        self.tx.cell_deps_iter()
    }

    /// Returns output Outpoints as an interator from the inner TX
    pub fn output_pts_iter(&self) -> impl Iterator<Item = crate::ckb_types::packed::OutPoint> {
        self.tx.output_pts_iter()
    }

    /// Returns input outpoints as an interator from the inner TX
    pub fn input_pts_iter(&self) -> impl Iterator<Item = crate::ckb_types::packed::OutPoint> {
        self.tx.input_pts_iter()
    }

    /// Returns outputs with data as an interator from the inner TX
    pub fn outputs_with_data_iter(&self) -> impl Iterator<Item = CellOutputWithData> {
        self.tx.outputs_with_data_iter()
    }

    /// Returns outputs total capacity from the inner TX
    pub fn outputs_capacity(
        &self,
    ) -> Result<crate::ckb_types::core::Capacity, ckb_types::core::CapacityError> {
        self.tx.outputs_capacity()
    }

    /// Sets the hash of the inner transaction to an unchecked one
    pub fn fake_hash(mut self, hash: crate::ckb_types::packed::Byte32) -> Self {
        self.tx = self.tx.fake_hash(hash);
        self
    }

    /// Sets a fake witness hash.
    pub fn fake_witness_hash(mut self, witness_hash: crate::ckb_types::packed::Byte32) -> Self {
        self.tx = self.tx.fake_witness_hash(witness_hash);
        self
    }
}

#[test]
fn test_conversions() {
    let _tx1 = Transaction::from(JsonTransactionView::default());
    let _tx2 = Transaction::from(JsonTransaction::default());
    let _tx3 = Transaction::from(PackedTransaction::default());
    let _tx4 = Transaction::from(TransactionBuilder::default().build());
    let _tx5 = Transaction::from(PackedTransactionView::default());
}
