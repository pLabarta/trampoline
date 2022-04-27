extern crate ckb_always_success_script;
extern crate trampoline_sdk;

use trampoline_sdk::chain::{MockChain, MockChainTxProvider as ChainRpc, Chain};
use trampoline_sdk::contract::*;
use trampoline_sdk::contract::{builtins::sudt::*, generator::*, schema::*};
use trampoline_sdk::types::{
    bytes::Bytes as TBytes,
    cell::{Cell, CellOutputWithData},
    script::Script as TScript,
    transaction::{Transaction, CellMetaTransaction}
};

use ckb_types::{
    bytes::Bytes,
    core::{TransactionBuilder, TransactionView},
    packed::{Byte32, CellOutput},
    prelude::*,
    H256,
};

use ckb_hash::blake2b_256;


use std::path::Path;

const EXPECTED_SUDT_HASH: &str =
    "0xe1e354d6d643ad42724d40967e334984534e0367405c5ae42a9d7d63d77df419";


fn gen_sudt_contract(
    minter_lock: TScript,
    initial_supply: Option<u128>,
) -> SudtTrampolineContract {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();

    let path_to_sudt_bin = Path::new(&out_dir).join("simple_udt");

    // Generate Sudt contract args & data field
    let minter_lock_hash = OwnerLockHash::new(minter_lock.calc_script_hash().pack().unpack());
    let init_supply = SudtAmount::new(initial_supply.unwrap_or_default());

    // Sudt compiled executable code
    let sudt_src: TBytes = ContractSource::load_from_path(path_to_sudt_bin).unwrap().into();


    // Generate trampoline contract from compiled code
    let mut sudt_contract = SudtTrampolineContract::from(sudt_src);
   
    // Set the contract args & cell data for the cell that *uses* this contract
    assert!(sudt_contract.set_caller_cell_args(minter_lock_hash).is_ok());
    assert!(sudt_contract.set_caller_cell_data(init_supply).is_ok());
    sudt_contract
}

fn generate_always_success_lock(
    args: Option<TBytes>,
) -> TScript {
    let data: TBytes = ckb_always_success_script::ALWAYS_SUCCESS.to_vec().into();
    let data_hash = data.hash_256();
    let mut script = TScript::default();
    script.set_code_hash(data_hash);
    script.set_args(args.unwrap_or_default());
    script
}

fn generate_simple_udt_cell(sudt_contract: &SudtTrampolineContract) -> CellOutput {
    sudt_contract.as_cell_output().unwrap()
}

fn generate_mock_tx(
    outputs: Vec<Cell>,
) -> TransactionView {
    let outputs_data = outputs.iter().map(|c| c.data()).collect::<Vec<_>>();
    let outputs = outputs.iter().map(|c| CellOutput::from(c)).collect::<Vec<_>>();
    TransactionBuilder::default()
        .outputs(outputs)
        .outputs_data(outputs_data.into_iter().map(|b| b.into()))
        .build()
}



#[test]
fn test_failed_issuance_tx_no_permissions() {
    let mut chain = MockChain::default();

    // Create always success lock cell and add to chain
    let minter_lock_code_cell_data: TBytes =
        ckb_always_success_script::ALWAYS_SUCCESS.to_vec().into();

    let minter_lock = Cell::with_data(minter_lock_code_cell_data);
    let minter_lock_cell = chain.create_cell((&minter_lock).into(), minter_lock.data().into());
    let mut minter_lock_script = TScript::from(&minter_lock);
    let mut non_minter_lock_script = TScript::from(&minter_lock);

    minter_lock_script.set_args(vec![1_u8]);
    non_minter_lock_script.set_args(vec![200_u8]);

    
    let non_minter_lock_hash = non_minter_lock_script.calc_script_hash();
    let minter_lock_hash = minter_lock_script.calc_script_hash();

    chain.create_cell(
        Cell::with_lock(non_minter_lock_script).into(),
        Default::default(),
    );

    // Deploy SUDT to chain
    let mut sudt_contract = gen_sudt_contract(minter_lock_script, Some(1500));

    let sudt_code_cell: Cell = sudt_contract.as_code_cell().unwrap();
    let sudt_code_cell_outpoint = chain.create_cell((&sudt_code_cell).into(), sudt_code_cell.data().into());

    // Create Mint SUDT transaction, using as input a cell locked with a different user's lock script
    // Should fail because the user does not have mint permissions
    sudt_contract.source = Some(ContractSource::Chain(sudt_code_cell_outpoint.into()));
    //let fail_tx = TransactionBuilder::default().build();

    // Add rule to sudt output generation to increase the amount field.
    // sudt_contract.add_output_rule(
    //     ContractField::Data,
    //     |ctx| -> ContractCellField<OwnerLockHash, SudtAmount> {
    //         let amount: ContractCellField<OwnerLockHash, SudtAmount> =
    //             ctx.load(ContractField::Data);
    //         if let ContractCellField::Data(amount) = amount {
    //             let amt: u128 = amount.into();
    //             ContractCellField::Data(SudtAmount::from(amt + 2000))
    //         } else {
    //             amount
    //         }
    //     },
    // );

    // sudt_contract.add_input_rule(move |_tx| -> CellQuery {
    //     CellQuery {
    //         _query: QueryStatement::Single(CellQueryAttribute::LockHash(
    //             non_minter_lock_hash.clone().into(),
    //         )),
    //         _limit: 1,
    //     }
    // });

    // // Instantiate chain rpc and tx generator
    // let chain_rpc = ChainRpc::new(chain);
    // let generator = Generator::new()
    //     .chain_service(&chain_rpc)
    //     .query_service(&chain_rpc)
    //     .pipeline(vec![&sudt_contract]);

    // let new_fail_tx = generator.generate(); //generator.pipe(fail_tx, Arc::new(Mutex::new(vec![])));
    //                                         // Test that failure transaction failed
    // let is_valid = chain_rpc.verify_tx(new_fail_tx.tx.into());
    // assert!(!is_valid);
}