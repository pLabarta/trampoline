use ckb_jsonrpc_types::{Byte32, Capacity, OutPoint, Script, TransactionView as JsonTransaction};
use std::prelude::v1::*;

use crate::ckb_types::{
    core::{TransactionBuilder, TransactionView, cell::CellMeta},
    packed::{CellInputBuilder, CellInput},
    prelude::*,
};

use std::sync::{Arc, Mutex};

use crate::chain::CellOutputWithData;

#[derive(Clone, Debug)]
pub struct CellMetaTransaction {
    pub tx: TransactionView,
    pub inputs: Vec<CellMeta>,
}

impl From<TransactionView> for CellMetaTransaction {
    fn from(tx: TransactionView) -> Self {
        Self {
            tx,
            inputs: vec![]
        }
    }
}

impl CellMetaTransaction {
    pub fn tx(self, tx: TransactionView) -> Self {
        Self {
            tx,
            inputs: self.inputs
        }
    }

    pub fn with_inputs(self, inputs: Vec<CellMeta>) -> Self {
        Self {
            tx: self.tx,
            inputs
        }
    }

    pub fn as_advanced_builder(&self) -> TransactionBuilder {
        self.tx.as_advanced_builder()
    }

    pub fn cell_deps(&self) -> crate::ckb_types::packed::CellDepVec {
        self.tx.cell_deps()
    }

    pub fn inputs(&self) -> crate::ckb_types::packed::CellInputVec {
        self.tx.inputs()
    }

    pub fn outputs(&self) -> crate::ckb_types::packed::CellOutputVec {
        self.tx.outputs()
    }

    pub fn outputs_data(&self) -> crate::ckb_types::packed::BytesVec {
        self.tx.outputs_data()
    }

    pub fn witnesses(&self) -> crate::ckb_types::packed::BytesVec {
        self.tx.witnesses()
    } 

    pub fn output(&self, idx: usize) -> Option<crate::ckb_types::packed::CellOutput> {
        self.tx.output(idx)
    }

    pub fn output_with_data(&self, idx: usize) -> Option<CellOutputWithData> {
        self.tx.output_with_data(idx)
    }

    pub fn output_pts(&self) -> Vec<crate::ckb_types::packed::OutPoint> {
        self.tx.output_pts()
    }

    pub fn cell_deps_iter(&self) -> impl Iterator<Item = crate::ckb_types::packed::CellDep> {
        self.tx.cell_deps_iter()
    }

    pub fn output_pts_iter(&self) -> impl Iterator<Item = crate::ckb_types::packed::OutPoint> {
        self.tx.output_pts_iter()
    }

    pub fn input_pts_iter(&self) -> impl Iterator<Item = crate::ckb_types::packed::OutPoint> {
        self.tx.input_pts_iter()
    }

    pub fn outputs_with_data_iter(&self) -> impl Iterator<Item = CellOutputWithData> {
        self.tx.outputs_with_data_iter()
    }

    pub fn outputs_capacity(&self) -> Result<crate::ckb_types::core::Capacity, ckb_types::core::CapacityError> {
        self.tx.outputs_capacity()
    }
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

// Note: Uses ckb_jsonrpc_types
pub trait TransactionProvider {
    fn send_tx(&self, tx: JsonTransaction) -> Option<Byte32>;

    fn verify_tx(&self, tx: JsonTransaction) -> bool;
}

// Note: Uses ckb_types::core::TransactionView; not ckb_jsonrpc_types::TransactionView
pub trait GeneratorMiddleware {
    fn pipe(
        &self,
        tx: CellMetaTransaction,
        query_register: Arc<Mutex<Vec<CellQuery>>>,
    ) -> CellMetaTransaction;

    fn update_query_register(&self, tx: CellMetaTransaction, query_register: Arc<Mutex<Vec<CellQuery>>>);
}

// TODO: implement from for CellQueryAttribute on json_types and packed types
#[derive(Debug, Clone)]
pub enum CellQueryAttribute {
    LockHash(Byte32),
    LockScript(Script),
    TypeScript(Script),
    MinCapacity(Capacity),
    MaxCapacity(Capacity),
}

#[derive(Debug, Clone)]
pub enum QueryStatement {
    Single(CellQueryAttribute),
    FilterFrom(CellQueryAttribute, CellQueryAttribute),
    Any(Vec<CellQueryAttribute>),
    All(Vec<CellQueryAttribute>),
}

#[derive(Debug, Clone)]
pub struct CellQuery {
    pub _query: QueryStatement,
    pub _limit: u64,
}

pub trait QueryProvider {
    fn query(&self, query: CellQuery) -> Option<Vec<OutPoint>>;
    fn query_cell_meta(&self, query: CellQuery) -> Option<Vec<CellMeta>>;
}

#[derive(Default)]
pub struct Generator<'a, 'b> {
    middleware: Vec<&'a dyn GeneratorMiddleware>,
    chain_service: Option<&'b dyn TransactionProvider>,
    query_service: Option<&'b dyn QueryProvider>,
    tx: Option<CellMetaTransaction>,
    query_queue: Arc<Mutex<Vec<CellQuery>>>,
}

impl<'a, 'b> Generator<'a, 'b> {
    pub fn new() -> Self {
        Generator {
            middleware: vec![],
            chain_service: None,
            query_service: None,
            tx: Some(TransactionBuilder::default().build().into()),
            query_queue: Arc::new(Mutex::new(vec![])),
        }
    }

    pub fn pipeline(mut self, pipes: Vec<&'a dyn GeneratorMiddleware>) -> Self {
        self.middleware = pipes;
        self
    }

    pub fn chain_service(mut self, chain_service: &'b dyn TransactionProvider) -> Self {
        self.chain_service = Some(chain_service);
        self
    }

    pub fn query_service(mut self, query_service: &'b dyn QueryProvider) -> Self {
        self.query_service = Some(query_service);
        self
    }

    pub fn query(&self, query: CellQuery) -> Option<Vec<CellMeta>> {
        let res = self.query_service.unwrap().query_cell_meta(query.clone());
        println!(
            "Res in generator.query for cell_query {:?} is {:?}",
            query, res
        );
        res
    }

    pub fn generate(&self) -> CellMetaTransaction {
        self.pipe(self.tx.as_ref().unwrap().clone(), self.query_queue.clone())
    }

   pub fn resolve_queries(&self, query_register: Arc<Mutex<Vec<CellQuery>>>) -> Vec<CellMeta>{
        query_register
            .lock()
            .unwrap()
            .iter().flat_map(|query| self.query(query.to_owned()).unwrap())
            .collect::<Vec<_>>()
    }
}


impl GeneratorMiddleware for Generator<'_, '_> {

   fn update_query_register(&self, tx: CellMetaTransaction, query_register: Arc<Mutex<Vec<CellQuery>>>) {
    self.middleware.iter().for_each(|m| {
        m.update_query_register(tx.clone(), query_register.clone())
    });
   }
    fn pipe(
        &self,
        tx: CellMetaTransaction,
        query_register: Arc<Mutex<Vec<CellQuery>>>,
    ) -> CellMetaTransaction {
     
        self.update_query_register(tx.clone(), query_register.clone());
        let inputs = self.resolve_queries(query_register.clone());
        println!("RESOLVED INPUTS IN GENERATOR PIPE: {:?}", inputs);
        let inner_tx = tx.as_advanced_builder()
            .set_inputs(inputs.iter().map(|inp| CellInputBuilder::default().previous_output(inp.out_point.clone()).build()).collect::<Vec<_>>())
            .build();
       let tx = tx.tx(inner_tx).with_inputs(inputs);
        let tx = self.middleware.iter().fold(tx, |tx, middleware| {
            middleware.pipe(tx, query_register.clone())
        });

        
        // TO DO: Resolve cell deps of inputs
        //         Will have to accommodate some cells being deptype of depgroup
        
       

        println!("FINAL TX GENERATED: {:#?}", ckb_jsonrpc_types::TransactionView::from(tx.clone().tx));
        tx

    }
}
