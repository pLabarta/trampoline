use anyhow::anyhow;
use anyhow::Result;
use ckb_app_config::{AppConfig, BlockAssemblerConfig, CKBAppConfig};
use ckb_hash::blake2b_256;
use ckb_system_scripts::*;
use ckb_types::prelude::Pack;
use ckb_types::{h160, h256, H160, H256};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str::FromStr;
use structopt::StructOpt;
use tera::{self, Context as TeraContext};
use trampoline::docker::*;
use trampoline::opts::{NetworkCommands, SchemaCommand, TrampolineCommand};
use trampoline::project::*;
use trampoline::schema::{Schema, SchemaError, SchemaInitArgs};
use trampoline::TrampolineResource;
use trampoline::{hex_decode, hex_encode, hex_string, parse_hex};
use trampoline::{TrampolineResourceType, TEMPLATES};

const SECP_TYPE_HASH: H256 =
    h256!("0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8");
fn create_block_assembler_from_pkhash(hash: &[u8]) -> BlockAssemblerConfig {
    use ckb_jsonrpc_types::{JsonBytes, ScriptHashType};
    BlockAssemblerConfig {
        code_hash: SECP_TYPE_HASH,
        hash_type: ScriptHashType::Type,
        use_binary_version_as_message_prefix: false,
        args: JsonBytes::from_bytes(bytes::Bytes::copy_from_slice(hash)),
        message: JsonBytes::default(),
        binary_version: "".to_string(),
    }
}

fn main() -> Result<()> {
    let opts = TrampolineCommand::from_args();

    let project = TrampolineProject::load(std::env::current_dir()?);

    match opts {
        TrampolineCommand::NewProject { name } => match project {
            Ok(project) => {
                if let TrampolineResourceType::Project(project) = project {
                    return Err(TrampolineProjectError::ProjectAlreadyExists {
                        name: project.config.name.to_string(),
                        path: project.root_dir.as_path().to_str().unwrap().to_string(),
                    })?;
                }
            }
            Err(e) => {
                let project = TrampolineProject::from(TrampolineProject::init(name)?);
                std::env::set_current_dir(&project.root_dir)?;
                Docker::default().build()?;
            }
        },
        TrampolineCommand::Schema { command } => {
            match command {
                SchemaCommand::New { name, def } => {
                    let mut schema_args: SchemaInitArgs =
                        (TrampolineProject::from(project?), name, None);
                    if def.is_some() {
                        let content = def.unwrap();
                        schema_args.2 = Some(content);
                    }

                    //println!("{:?}", schema_args);
                    let result = Schema::init(schema_args)?;
                }
                SchemaCommand::Build { name } => {
                    let mut schema_args: SchemaInitArgs =
                        (TrampolineProject::from(project?), name, None);
                    let result = Schema::init(schema_args)?;
                }
            }
        }
        TrampolineCommand::Network { command } => {
            let project = TrampolineProject::from(project?);
            match command {
                // TODO: Init and Run as separate so that set miner does not require launching network first
                NetworkCommands::Launch {} => {
                    let miner_config = &project.config.env.as_ref().unwrap().miner;
                    let miner_ports = (
                        miner_config.host_port.into(),
                        miner_config.container_port.into(),
                    );
                    Docker::default()
                        .name(project.config.name.as_str())
                        .add_service(project.config.env.unwrap().chain)?
                        .run(
                            Some(vec!["run".to_string()]),
                            Some("node"),
                            vec![miner_ports],
                        )?;
                }
                NetworkCommands::SetMiner { pubkey, lock_arg } => {
                    let mut config = project.load_ckb_config()?;
                    if let Some(pubkey) = pubkey {
                        let pubkey_bytes = parse_hex(pubkey.as_str())?;
                        let pubkey_hash = blake2b_256(&pubkey_bytes);
                        let pkey_hash_slice = &pubkey_hash[0..20];
                        let block_assembler = create_block_assembler_from_pkhash(pkey_hash_slice);
                        config.block_assembler = Some(block_assembler);
                        project.save_ckb_config(config)?;
                    } else if let Some(lock_arg) = lock_arg {
                        let lock_arg_bytes = parse_hex(&lock_arg.as_str())?;
                        let block_assembler = create_block_assembler_from_pkhash(&lock_arg_bytes);
                        config.block_assembler = Some(block_assembler);
                        project.save_ckb_config(config)?;
                    }
                    Docker::default()
                        .name(format!("{}-node", project.config.name.as_str()).as_str())
                        .restart()?;
                }
                NetworkCommands::Miner { one_block: _ } => {
                    let config = project.load_ckb_config()?;
                    let block_assembler_args = config.block_assembler.as_ref();
                    if block_assembler_args.is_none() {
                        return Err(anyhow!("No miner address set. Refer to `trampoline net set-miner --help` for more information."));
                    }
                    let container_name = project.config.name.as_str();
                    let miner_mount_path = &project.config.env.unwrap().miner.container_mount;
                    Docker::exec(
                        format!("{}-node", container_name).as_str(),
                        vec!["ckb", "miner"],
                        miner_mount_path,
                    )?;
                }
                NetworkCommands::Indexer {} => {}
                _ => {}
            }
        }
        _ => {
            println!("Unrecognized command. Use `trampoline --help` for usage information.")
        }
    }

    Ok(())
}
