//! Types for creating custom queries for CKB Indexer

use ckb_jsonrpc_types::{Byte32, Capacity, Script};
use std::prelude::v1::*;

// TODO: implement from for CellQueryAttribute on json_types and packed types
/// Query attributes used to define QueryStatements
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CellQueryAttribute {
    /// The cell's lock hash
    LockHash(Byte32),
    /// The cell's lock script
    LockScript(Script),
    /// The cell's type script
    TypeScript(Script),
    /// Minimum capacity the cell must have
    MinCapacity(Capacity),
    /// Maximum capacity the cell must have
    MaxCapacity(Capacity),
    /// The hash of the cell's data
    DataHash(Byte32),
}

/// Statement used to filter cells from a collection
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QueryStatement {
    /// Match only one query attribute
    Single(CellQueryAttribute),
    /// Match all cells with the first attribute and filter the second one
    FilterFrom(CellQueryAttribute, CellQueryAttribute),
    /// Match all cells that have one of the attributes
    Any(Vec<CellQueryAttribute>),
    /// Match cells that have every attribute
    All(Vec<CellQueryAttribute>),
}

/// Query type that containts a statement and an amount of cells
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CellQuery {
    /// Query statement that cells should match
    pub _query: QueryStatement,
    /// Maximum amount of cells that should be returned
    pub _limit: u64,
}
