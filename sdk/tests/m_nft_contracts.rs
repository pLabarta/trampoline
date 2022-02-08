extern crate trampoline_sdk;
use ckb_always_success_script::ALWAYS_SUCCESS;

use trampoline_sdk::chain::{MockChain, MockChainTxProvider as ChainRpc};
use trampoline_sdk::contract::*;
use trampoline_sdk::contract::{builtins::m_nft::*, generator::*};

use ckb_types::{
    bytes::Bytes,
    core::{TransactionBuilder, TransactionView},
    packed::{Byte32, CellOutput},
    prelude::*,
    H256,
};

use ckb_hash::blake2b_256;
use ckb_jsonrpc_types::JsonBytes;

use ckb_types::packed::{CellInput, CellInputBuilder, CellOutputBuilder};
use std::path::Path;
use std::sync::{Arc, Mutex};
use ckb_verification::HeaderErrorKind::Version;

fn gen_issuer_contract(seed_input: Option<CellInput>) -> MultiNFTIssuerContract {
    let out_dir = std::env::var_os("OUT_DIR").unwrap();

    let path_to_issuer_bin = Path::new(&out_dir).join("m_nft/issuer-type");

    let src = ContractSource::load_from_path(path_to_issuer_bin).unwrap();

    MultiNFTIssuerContract {
        source: Some(ContractSource::Immediate(src.clone())),
        data: mNFTIssuer::default(),
        args: {
            if let Some(seed) = &seed_input {
                NftIssuerArgs::from_cell_input(seed, 0)
            } else {
                NftIssuerArgs::default()
            }
        },
        lock: None,
        type_: None,
        code: Some(JsonBytes::from_bytes(src)),
        output_rules: vec![],
        input_rules: vec![],
    }
}

#[test]
fn test_issuer_create() {
    let mut chain = MockChain::default();

    let always_success_cell =
        chain.deploy_cell_with_data(Bytes::copy_from_slice(ALWAYS_SUCCESS.as_slice()).into());

    let seed_cell_lock = chain
        .build_script(&always_success_cell, Bytes::default())
        .unwrap();

    let seed_cell_outpoint = chain.create_cell(
        CellOutput::new_builder()
            .capacity(5000_u64.pack())
            .lock(seed_cell_lock.clone())
            .build(),
        Bytes::default(),
    );

    let mut issuer_contract = gen_issuer_contract(Some(
        CellInputBuilder::default()
            .previous_output(seed_cell_outpoint.clone())
            .build(),
    ));

    let issuer_code_cell = issuer_contract.as_code_cell();
    let deployed_issuer_code_cell = chain.create_cell(issuer_code_cell.0, issuer_code_cell.1);

    let issuer_instance = CellOutputBuilder::default()
        .lock(seed_cell_lock.clone())
        .type_(
            Some(ckb_types::packed::Script::from(
                issuer_contract.as_script().unwrap(),
            ))
            .pack(),
        )
        .capacity(1000_u64.pack())
        .build();

    let issuance_tx = TransactionBuilder::default()
        .cell_dep(
            issuer_contract
                .as_cell_dep(deployed_issuer_code_cell.into())
                .into(),
        )
        .cell_dep(chain.find_cell_dep_for_script(&seed_cell_lock))
        .output(issuer_instance)
        .input(
            CellInputBuilder::default()
                .previous_output(seed_cell_outpoint)
                .build(),
        )
        .output_data(Bytes::copy_from_slice([0u8; 11].as_slice()).pack())
        .build();

    issuer_contract.add_output_rule(
        ContractCellFieldSelector::Data,
        |issuer_data: ContractCellField<NftIssuerArgs, mNFTIssuer>|
            -> ContractCellField<NftIssuerArgs, mNFTIssuer> {
            if let ContractCellField::Data(issuer) = issuer_data {
                ContractCellField::Data(mNFTIssuer::new(NftIssuer {
                    version: trampoline_sdk::contract::builtins::m_nft::Version::new(0_u8),
                    class_count: ClassCount::new(0),
                    set_count: SetCount::new(0),
                    info_size: InfoSize::new(0),
                    info: Default::default()
                }))
            } else {
                issuer_data
            }
        }
    );

    issuer_contract.add_input_rule(move |_tx| -> CellQuery {
        CellQuery {
            _query: QueryStatement::Single(CellQueryAttribute::LockHash(
                seed_cell_lock.calc_script_hash().into(),
            )),
            _limit: 1,
        }
    });

    let chain_rpc = ChainRpc::new(chain);

    let generator = Generator::new()
        .chain_service(&chain_rpc)
        .query_service(&chain_rpc)
        .pipeline(vec![&issuer_contract]);

    let issuer_create_tx = generator.pipe(issuance_tx, Arc::new(Mutex::new(vec![])));

    let is_valid = chain_rpc.verify_tx(issuer_create_tx.into());
    assert!(is_valid);
}
