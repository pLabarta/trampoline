
pub mod builtins;
pub mod schema;
use self::schema::*;

use crate::ckb_types::packed::{CellOutput, CellOutputBuilder, Uint64};
use crate::ckb_types::{bytes::Bytes, packed, prelude::*};


#[cfg(not(feature = "script"))]
pub mod generator;
#[cfg(not(feature = "script"))]
use self::generator::{CellQuery, GeneratorMiddleware};
#[cfg(not(feature = "script"))]
use crate::chain::CellOutputWithData;
#[cfg(not(feature = "script"))]
use ckb_hash::blake2b_256;
#[cfg(not(feature = "script"))]
use ckb_jsonrpc_types::{CellDep, DepType, JsonBytes, OutPoint, Script};
use std::prelude::v1::*;
#[cfg(not(feature = "script"))]
use std::fs;
#[cfg(not(feature = "script"))]
use std::path::PathBuf;
#[cfg(not(feature = "script"))]
use std::sync::{Arc, Mutex};
#[cfg(not(feature = "script"))]
use crate::ckb_types::H256;
#[cfg(not(feature = "script"))]
use crate::ckb_types::core::TransactionView;




#[cfg(not(feature = "script"))]
#[derive(Debug, Clone)]
pub enum ContractSource {
    LocalPath(PathBuf),
    Immediate(Bytes),
    Chain(OutPoint),
}
#[cfg(not(feature = "script"))]
impl ContractSource {
    pub fn load_from_path(path: PathBuf) -> std::io::Result<Bytes> {
        let file = fs::read(path)?;
        println!("SUDT CODE SIZE: {}", file.len());
        Ok(Bytes::from(file))
    }
}

#[cfg(not(feature = "script"))]
pub enum ContractCellFieldSelector {
    Args,
    Data,
    LockScript,
    TypeScript,
    Capacity,
}
#[cfg(not(feature = "script"))]
pub enum ContractCellField<A, D> {
    Args(A),
    Data(D),
    LockScript(ckb_types::packed::Script),
    TypeScript(ckb_types::packed::Script),
    Capacity(Uint64),
}
#[cfg(not(feature = "script"))]
#[derive(Default)]
#[cfg(not(feature = "script"))]
pub struct Contract<A, D> {
    pub source: Option<ContractSource>,
    pub data: D,
    pub args: A,
    pub lock: Option<Script>,
    pub type_: Option<Script>,
    pub code: Option<JsonBytes>,
    #[allow(clippy::type_complexity)]
    pub output_rules: Vec<Box<dyn Fn(ContractCellField<A, D>) -> ContractCellField<A, D>>>,
    pub input_rules: Vec<Box<dyn Fn(TransactionView) -> CellQuery>>,
}

#[cfg(not(feature = "script"))]
impl<A, D> Contract<A, D>
where
    D: JsonByteConversion + MolConversion + BytesConversion + Clone,
    A: JsonByteConversion + MolConversion + BytesConversion + Clone,
{
    // The lock script of the cell containing contract code
    pub fn lock(mut self, lock: Script) -> Self {
        self.lock = Some(lock);
        self
    }

    // The type script of the cell containing contract code
    pub fn type_(mut self, type_: Script) -> Self {
        self.type_ = Some(type_);
        self
    }

    pub fn data_hash(&self) -> Option<H256> {
        if let Some(data) = &self.code {
            let byte_slice = data.as_bytes();

            let raw_hash = blake2b_256(&byte_slice);
            H256::from_slice(&raw_hash).ok()
        } else {
            None
        }
    }

    // Returns a script structure which can be used as a lock or type script on other cells.
    // This is an easy way to let other cells use this contract
    pub fn as_script(&self) -> Option<ckb_jsonrpc_types::Script> {
        self.data_hash().map(|data_hash| {
            Script::from(
                packed::ScriptBuilder::default()
                    .args(self.args.to_bytes().pack())
                    .code_hash(data_hash.pack())
                    .hash_type(ckb_types::core::ScriptHashType::Data1.into())
                    .build(),
            )
        })
    }

    // pub fn as_script_with_type_hash(&self) -> Option<ckb_jsonrpc_types::Script> {
    //     // To do: check is hash_type_type
    //     let script_hash = self.as_code_cell().0.type_().to_opt().unwrap().calc_script_hash().into();
    //
    //     Some(Script::from(
    //         packed::ScriptBuilder::default()
    //             .args(self.args.to_bytes().pack())
    //             .code_hash(script_hash.pack())
    //             .hash_type(ckb_types::core::ScriptHashType::Type.into())
    //             .build()
    //     ))
    // }

    // Return a CellOutputWithData which is the code cell storing this contract's logic
    pub fn as_code_cell(&self) -> CellOutputWithData {
        let data: Bytes = self.code.clone().unwrap_or_default().into_bytes();
        let type_script = self.type_.clone().unwrap_or_default();
        let type_script = {
            if self.type_.is_some() {
                Some(ckb_types::packed::Script::from(type_script))
            } else {
                None
            }
        };

        let cell_output = CellOutputBuilder::default()
            .capacity((data.len() as u64).pack())
            .lock(self.lock.clone().unwrap_or_default().into())
            .type_(type_script.pack())
            .build();
        (cell_output, data)
    }

    pub fn script_hash(&self) -> Option<ckb_jsonrpc_types::Byte32> {
        let script: ckb_types::packed::Script = self.as_script().unwrap().into();
        Some(script.calc_script_hash().into())
    }

    pub fn as_cell_dep(&self, out_point: OutPoint) -> CellDep {
        CellDep {
            out_point,
            dep_type: DepType::Code,
        }
    }

    // Set data of a cell that will *reference* (i.e., use) this contract
    pub fn set_raw_data(&mut self, data: impl Into<JsonBytes>) {
        self.data = D::from_json_bytes(data.into());
    }

    pub fn set_data(&mut self, data: D) {
        self.data = data;
    }

    // Set args of a cell that will *reference* (i.e., use) this contract
    pub fn set_raw_args(&mut self, args: impl Into<JsonBytes>) {
        self.args = A::from_json_bytes(args.into());
    }

    pub fn set_args(&mut self, args: A) {
        self.args = args;
    }

    pub fn read_data(&self) -> D {
        self.data.clone()
    }

    pub fn read_args(&self) -> A {
        self.args.clone()
    }

    pub fn read_raw_data(&self, data: Bytes) -> D {
        D::from_bytes(data)
    }

    pub fn read_raw_args(&self, args: Bytes) -> A {
        A::from_bytes(args)
    }

    pub fn add_output_rule<F>(&mut self, transform_func: F)
    where
        F: Fn(ContractCellField<A, D>) -> ContractCellField<A, D> + 'static,
    {
        self.output_rules.push((Box::new(transform_func)));
    }

    pub fn add_input_rule<F>(&mut self, query_func: F)
    where
        F: Fn(TransactionView) -> CellQuery + 'static,
    {
        self.input_rules.push(Box::new(query_func))
    }
}
#[cfg(not(feature = "script"))]
impl<A, D> GeneratorMiddleware for Contract<A, D>
where
    D: JsonByteConversion + MolConversion + BytesConversion + Clone,
    A: JsonByteConversion + MolConversion + BytesConversion + Clone,
{
    fn pipe(
        &self,
        tx: TransactionView,
        query_queue: Arc<Mutex<Vec<CellQuery>>>,
    ) -> TransactionView {
        type OutputWithData = (CellOutput, Bytes);
        let mut idx = 0;
        let outputs = tx.clone().outputs().into_iter().filter_map(|output| {
            let self_script_hash: ckb_types::packed::Byte32 = self.script_hash().unwrap().into();

            if let Some(type_) = output.type_().to_opt() {
                if type_.calc_script_hash() == self_script_hash {
                    return tx.output_with_data(idx);
                }
            }

            if output.lock().calc_script_hash() == self_script_hash {
                return tx.output_with_data(idx);
            }

            idx += 1;
            None
        });

        let outputs = outputs
            .into_iter()
            .map(|output| {
                let processed =
                    self.output_rules
                        .iter()
                        .fold(output, |output, rule| {
                                let data = self.read_raw_data(output.1.clone());
                                println!("Data before update {:?}", data.to_mol());
                                let updated_field = rule(ContractCellField::Data(data));
                                if let ContractCellField::Data(new_data) = updated_field {
                                    println!("Data after update {:?}", new_data.to_mol());

                                    (output.0, new_data.to_bytes())
                                } else {
                                    output
                                }
                            }
                         );
                println!("Output bytes of processed output: {:?}", processed.1.pack());
                processed
            })
            .collect::<Vec<OutputWithData>>();

        let queries = self.input_rules.iter().map(|rule| rule(tx.clone()));

        query_queue.lock().unwrap().extend(queries);

        tx.as_advanced_builder()
            .set_outputs(
                outputs
                    .iter()
                    .map(|out| out.0.clone())
                    .collect::<Vec<CellOutput>>(),
            )
            .set_outputs_data(
                outputs
                    .iter()
                    .map(|out| out.1.clone().pack())
                    .collect::<Vec<ckb_types::packed::Bytes>>(),
            )
            .build()
    }
}
