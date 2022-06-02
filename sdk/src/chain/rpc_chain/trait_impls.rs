use super::*;
use crate::chain::Chain;
use provider::RpcProvider;

impl Chain for RpcChain {
    type Inner = RpcProvider;

    fn inner(&self) -> Self::Inner {
        RpcProvider::new(self.clone())
    }

    fn deploy_cell(
        &mut self,
        cell: &crate::types::cell::Cell,
        unlockers: crate::chain::Unlockers,
        inputs: &crate::chain::CellInputs,
    ) -> crate::chain::ChainResult<OutPoint> {
        todo!()
    }

    fn deploy_cells(
        &mut self,
        cells: &Vec<crate::types::cell::Cell>,
        unlockers: crate::chain::Unlockers,
        inputs: &crate::chain::CellInputs,
    ) -> crate::chain::ChainResult<Vec<OutPoint>> {
        todo!()
    }

    fn set_default_lock<A, D>(&mut self, lock: crate::contract::Contract<A, D>)
    where
        D: crate::contract::schema::JsonByteConversion
            + crate::contract::schema::MolConversion
            + crate::contract::schema::BytesConversion
            + Clone
            + Default,
        A: crate::contract::schema::JsonByteConversion
            + crate::contract::schema::MolConversion
            + crate::contract::schema::BytesConversion
            + Clone
            + Default,
    {
        todo!()
    }

    fn generate_cell_with_default_lock(
        &self,
        lock_args: crate::types::bytes::Bytes,
    ) -> crate::types::cell::Cell {
        todo!()
    }
}
