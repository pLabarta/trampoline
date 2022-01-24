use serde::Deserialize;
use serde::Serialize;
use std::path::Path;
use std::path::PathBuf;

use ckb_app_config::ChainConfig;
use ckb_app_config::LogConfig;
use ckb_app_config::MinerAppConfig;

#[derive(Serialize, Deserialize)]
enum NetworkService {
    Node {
        name: String,
        rpc_port: Option<String>,
    },
    Miner {
        name: String,
        node: String,
    },
    Indexer {
        name: String,
        node: String,
    },
}

#[derive(Serialize, Deserialize)]
enum NetworkMode {
    Main,
    Test,
    Dev,
}

#[derive(Serialize, Deserialize)]
pub struct Network {
    name: String,
    services: Vec<NetworkService>,
    env: NetworkMode,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    name: String,
    dev_lockarg: String,
}

impl Network {
    pub fn from_config(path: &str) {
        let path = Path::new(path);
        println!("From Config function from the config module.")
    }

    pub fn save_toml(&self, path: &str) {
        let toml = toml::to_string(&self).unwrap();
        std::fs::write("network.toml", toml).expect("Unable to write file.");
    }
}

pub fn default_miner_config() -> MinerAppConfig {
    let dummy_config = ckb_app_config::DummyConfig::Constant { value: 5000 };
    let worker = ckb_app_config::MinerWorkerConfig::Dummy(dummy_config);

    MinerAppConfig {
        sentry: ckb_app_config::SentryConfig {
            dsn: "".to_string(),
            org_contact: None,
            org_ident: None
        },
        bin_name: "ckb".to_string(),
        chain: ChainConfig {
            spec: ckb_resource::Resource::file_system(PathBuf::from("specs/dev.toml")),
        },
        data_dir: PathBuf::from("data"),
        logger: LogConfig::default(),
        memory_tracker: ckb_app_config::MemoryTrackerConfig::default(),
        metrics: ckb_app_config::MetricsConfig::default(),
        miner: ckb_app_config::MinerConfig {
            client: ckb_app_config::MinerClientConfig {
                block_on_submit: true,
                poll_interval: 1000,
                rpc_url: "http://0.0.0.0:8114".to_string(),
            },
            workers: vec![worker],
        },
        root_dir: PathBuf::from("/var/lib/ckb"),
    }
}


pub fn default_ckb_config() -> ckb_app_config::CKBAppConfig {

    ckb_app_config::CKBAppConfig {
        bin_name: "ckb".to_string(),
        data_dir: PathBuf::from("data"),
        block_assembler: Some(ckb_app_config::BlockAssemblerConfig {
            code_hash: "0x9bd7e06f3ecf4be0f2fcd2188b23f1b9fcc88e5d4b65a8637b17723bbda3cce8",
            
        })

    }
}