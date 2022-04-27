use super::types::*;
use super::generator::{CellQuery, GeneratorMiddleware};
use crate::types::cell::{Cell, CellError, CellResult};
use crate::types::bytes::{Bytes as TBytes};
use crate::types::script::Script as TScript;
use crate::ckb_types::packed::{CellInput, CellOutput, CellOutputBuilder, Uint64};
use crate::ckb_types::{bytes::Bytes, packed, prelude::*};
use crate::types::{
    transaction::CellMetaTransaction,
    cell::CellOutputWithData,
};




use crate::ckb_types::core::TransactionView;

use crate::ckb_types::{core::TransactionBuilder, H256};

use ckb_hash::blake2b_256;

use ckb_jsonrpc_types::{CellDep, DepType, JsonBytes, OutPoint, Script};
use ckb_types::core::cell::CellMeta;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum TContractError {
    #[error(transparent)]
    CellError(#[from] CellError),
}
pub type TContractResult<T> = Result<T, TContractError>;
// Replacement for Contract

pub struct TContract<A: Default, D: Default> {
    pub source: Option<ContractSource>,
    inner_code_cell: Cell,
    inner_usage_cell: Cell,
    pub output_rules: Vec<OutputRule<A, D>>,
    pub input_rules: Vec<Box<dyn Fn(TransactionView) -> CellQuery>>,
    pub outputs_count: usize,
    pub contract_type: ContractType,
}

impl<A, D> Default for TContract<A, D>
where
    A: Default + Into<TBytes>,
    D: Default + Into<TBytes>,
{
    fn default() -> Self {
        Self {
            source: Default::default(),
            inner_code_cell: Default::default(),
            inner_usage_cell: Default::default(),
            output_rules: Default::default(),
            input_rules: Default::default(),
            outputs_count: 1,
            contract_type: ContractType::Type,
        }
    }
}


impl<A, D> From<TBytes> for TContract<A, D>
where
    A: Default,
    D: Default,
{
    
    fn from(bytes: TBytes) -> Self {
        Self {
            source: Some(ContractSource::Immediate((&bytes).into())),
            inner_code_cell: Cell::with_data(bytes),
            inner_usage_cell: Default::default(),
            output_rules: Default::default(),
            input_rules: Default::default(),
            outputs_count: 1,
            contract_type: ContractType::Type,
        }
    }
}

enum InnerCellType {
    Code,
    Caller
}

impl<A,D> TContract<A,D>
where
    A: Into<TBytes> + Default,
    D: Into<TBytes> + Default,
{

    fn safe_cell_update(&mut self, cell: Cell, cell_type: InnerCellType) -> TContractResult<()> {
        // In the case where the code cell hash been updated, ensure that
        // the usage cell's relevant script (lock or type) uses the correct code_hash
        match cell_type {
            InnerCellType::Caller => {
                self.inner_usage_cell = cell;
                Ok(())
            },
            InnerCellType::Code => {
               let new_code_hash = cell.data_hash();
               self.inner_code_cell = cell;
               match self.contract_type {
                    ContractType::Type => {
                        let script = self.inner_usage_cell.type_script()?;
                        if let Some(mut script) = script {
                            script.set_code_hash(new_code_hash);
                            self.inner_usage_cell.set_type_script(script).map_err(|e| e.into())
                        } else {
                            Ok(())
                        }
                    
                    },
                    ContractType::Lock => {
                        let mut script = self.inner_usage_cell.lock_script()?;
                        script.set_code_hash(new_code_hash);
                        self.inner_usage_cell.set_lock_script(script).map_err(|e| e.into())
                    },
                }
            }
        }
    }
    fn update_inner_cells<F>(&mut self, update: F, cell_type: InnerCellType)  -> TContractResult<()> 
        where F: FnOnce(Cell) -> CellResult<Cell>, 
    {
        let cell_to_update = match &cell_type {
            InnerCellType::Caller => {
                update(self.inner_usage_cell.clone())
            },
            InnerCellType::Code => {
                update(self.inner_code_cell.clone())
            }
        }?;
        self.safe_cell_update(cell_to_update, cell_type)?;
        // check if self.script_hash() == self.inner_usage_cell.lock_script_hash() or type_script_hash()
        Ok(())
    }

    // unfortunate clones here
    pub fn set_lock(&mut self, lock: impl Into<TScript>) -> TContractResult<()>{
        //let lock: TScript = lock.into();
        self.update_inner_cells(|mut cell| {
            //let mut cell = cell.clone();
            cell.set_lock_script(lock)?;
            Ok(cell)
        }, InnerCellType::Code)
    }

    pub fn set_type(&mut self, type_: impl Into<TScript>) -> TContractResult<()> {
       // let type_ = type_.into();
        self.update_inner_cells(|mut cell| {
            //let mut cell = cell.clone();
            cell.set_type_script(type_)?;
            Ok(cell)
        }, InnerCellType::Code)
    }

    pub fn set_caller_cell_data(&mut self, data: D) -> TContractResult<()> {
        // let data:TBytes = data.into();
        self.update_inner_cells(move |cell| {
            let mut cell = cell.clone();
            cell.set_data(data)?;
            Ok(cell)
        }, InnerCellType::Caller)
    }

    pub fn set_caller_cell_args(&mut self, args: A) -> TContractResult<()> {
        match self.contract_type {
            ContractType::Type => {
               self.inner_usage_cell.set_type_args(args)?;
               Ok(())
                
            },
            ContractType::Lock => {
                self.inner_usage_cell.set_lock_args(args)?;
                Ok(())
            },
        }
    }

    pub fn code_hash(&self) -> H256 {
        self.inner_code_cell.data_hash()
    }


    pub fn script_hash(&self) ->Option<H256> {
        match self.contract_type {
            ContractType::Type => {
                self.inner_usage_cell.type_hash().ok().unwrap_or_default()            
            },
            ContractType::Lock => {
                self.inner_usage_cell.lock_hash().ok()
            },
        }        
    }

    pub fn caller_cell_data_hash(&self) -> H256 {
        self.inner_usage_cell.data_hash()
    }

    // Can be used to retrieve cell output, cell output with data, cell input, etc...
    pub fn as_caller_cell<C: From<Cell>>(&self) -> TContractResult<C> {
        let cell = self.inner_usage_cell.clone();
        cell.validate()?;
        Ok(cell.into())
    }

    pub fn as_code_cell<C: From<Cell>>(&self) -> TContractResult<C> {
        let cell = self.inner_code_cell.clone();
        cell.validate()?;
        Ok(cell.into())
    }


    // Can be used to retrieve packed script or json script struct
    pub fn as_script<S: From<TScript>>(&self) -> TContractResult<Option<S>> 
    {
        match self.contract_type {
            ContractType::Type => {
                Ok(self.inner_usage_cell.type_script()?.map(|s| s.into()))
                
            },
            ContractType::Lock => {
                Ok(Some(self.inner_usage_cell.lock_script()?.into()))
            },
        }
    }

    // Get code cell as a cell dep
    pub fn as_code_cell_dep(&self) -> TContractResult<ckb_types::packed::CellDep> {
        Ok(self.inner_code_cell.as_cell_dep(ckb_types::core::DepType::Code)?)    
    }

    // Get caller cell as a cell output
    pub fn as_cell_output(&self) -> TContractResult<CellOutput> {
        Ok(CellOutput::from(&self.inner_usage_cell))
    }
}
