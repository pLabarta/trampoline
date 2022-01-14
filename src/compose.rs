use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;

use ckb_app_config::CKBAppConfig;
use ckb_app_config::MinerAppConfig;

use serde::Deserialize;
use serde::Serialize;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct File {
    version: String,
    services: BTreeMap<String, Service>,
    #[serde(skip_serializing_if = "Option::is_none")]
    volumes: Option<BTreeMap<String, VolumeSetup>>,
}

impl File {
    pub fn from(services: Vec<Service>) -> Self {
        let mut servs: BTreeMap<String, Service> = BTreeMap::new();
        let version = "3".to_string();
        for service in &services {
            servs.insert(service.name.clone(), service.clone());
        }

        let mut vols: Vec<Volume> = vec![];
        for service in services {
            match service.volumes {
                None => {}
                Some(vol_list) => {
                    for vol in vol_list {
                        match vol.volume_type {
                            VolumeType::Bind => {}
                            VolumeType::Volume => vols.push(vol),
                        }
                    }
                }
            }
        }
        if vols.len() > 0 {
            let mut volumes = BTreeMap::new();
            for vol in vols {
                volumes.insert(vol.source, VolumeSetup::default());
            }
            File {
                version,
                services: servs,
                volumes: Some(volumes),
            }
        } else {
            File {
                version,
                services: servs,
                volumes: None,
            }
        }
    }

    pub fn hello() -> Self {
        let service = Service {
            name: "hello-world".to_string(),
            image: "hello-world".to_string(),
            volumes: None,
            expose: None,
            command: None,
            environment: None,
            ports: None,
            entrypoint: None,
            depends_on: None,
        };

        let services = vec![service];

        File::from(services)
    }

    pub fn write_yaml(file_name: String) {}

    pub fn test_module() {
        println!("This is printing from the compose module!");
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Service {
    #[serde(skip)]
    name: String,
    image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    volumes: Option<Vec<Volume>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expose: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    environment: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ports: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    entrypoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    depends_on: Option<Vec<String>>,
}

// Premade services
impl Service {
    pub fn hello() -> Self {
        Service {
            name: "hello-world".to_string(),
            image: "hello-world".to_string(),
            expose: None,
            volumes: None,
            command: None,
            environment: None,
            ports: None,
            entrypoint: None,
            depends_on: None,
        }
    }

    pub fn node(name: &str, rpc_port: Option<String>, dev: Option<bool>) -> Self {
        let development_mode = dev.unwrap_or(true);
        let chain_data = Volume {
            volume_type: VolumeType::Volume,
            source: format!("{}-node-chain-data", name),
            target: format!("/var/lib/ckb"),
        };
        let rpc_port_for_host = rpc_port.unwrap_or("8114".to_string());
        let port_string = format!("{}:8114", rpc_port_for_host);

        Service {
            name: format!("{}-node", name),
            image: "nervos/ckb".to_string(),
            expose: Some(vec!["8114".to_string(), "8115".to_string()]),
            volumes: Some(vec![chain_data]),
            command: Some("run".to_string()),
            environment: match development_mode {
                true => {
                    let env = "CKB_CHAIN=dev".to_string();
                    Some(vec![env])
                }

                false => None,
            },
            ports: Some(vec![port_string]),
            entrypoint: None,
            depends_on: None,
        }
    }

    pub fn miner(name: &str, dev: Option<bool>, from_node: &Service, cfg_path: &str) -> Self {
        let development_mode = dev.unwrap_or(true);
        let chain_data = Volume {
            volume_type: VolumeType::Volume,
            source: format!("{}-chain-data", &from_node.name),
            target: format!("/var/lib/ckb"),
        };

        let config_file = Volume {
            volume_type: VolumeType::Bind,
            source: cfg_path.to_string(),
            target: format!("/var/lib/ckb/ckb-miner.toml"),
        };

        Service {
            name: format!("{}-miner-{}", &from_node.name, name),
            image: "nervos/ckb".to_string(),
            expose: Some(vec!["8114".to_string(), "8115".to_string()]),
            volumes: Some(vec![chain_data, config_file]),
            command: Some("miner".to_string()),
            environment: match development_mode {
                true => {
                    let env = "CKB_CHAIN=dev".to_string();
                    Some(vec![env])
                }

                false => None,
            },
            ports: None,
            entrypoint: Some("cat /var/lib/ckb/ckb-miner.toml".to_string()),
            depends_on: Some(vec![from_node.name.clone()]),
        }
    }
}

fn setup_miner_config(miner_name: &str, node_name: &str) {
    let folder_string = format!(".trampoline/{}", &miner_name);
    let folder = Path::new(&folder_string);
    let file_string = format!("{}/ckb-miner.toml", &folder_string);
    let file = Path::new(&file_string);
    match (folder.exists(), file.exists()) {
        (true, true) => {
            println!("Miner configuration found at {:?}", &folder);
        }
        (true, false) => {
            println!(
                "Found directory for {} config, but no config file. One will be created at {}",
                &miner_name, &file_string
            );
        }
        _ => {
            println!("Creating new config for miner at {}", &file_string);
            // load config template
            let template_path_string = format!(".trampoline/network/{}", &miner_name);
            let template_path = Path::new(&template_path_string);
            let template = std::fs::read_to_string(template_path).unwrap();
            let mut config = MinerAppConfig::load_from_slice(template.as_bytes())
                .expect("Error loading template config.");
            // make changes, this only changes the url to use the docker one
            config.miner.client.rpc_url = format!("http://{}:8114", &node_name);

            // save config
            let config_toml =
                toml::Value::try_from(config).expect("Failed converting config to TOML");
            let config_string =
                toml::to_string(&config_toml).expect("Failed converting config TOML to string");
            let mut new_config_file = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(file)
                .expect("Error opening new config file");

            new_config_file
                .write_all(config_string.as_bytes())
                .expect("Failed writing new config to file");
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct VolumeSetup {}

impl VolumeSetup {
    pub fn default() -> Self {
        VolumeSetup {}
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
enum VolumeType {
    #[serde(rename(serialize = "volume", deserialize = "volume"))]
    Volume,
    #[serde(rename(serialize = "bind", deserialize = "bind"))]
    Bind,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
struct Volume {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    volume_type: VolumeType,
    source: String,
    target: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_from_single_node() {
        let node = Service::node("test", None, None);
        let service_list = vec![node.clone()];
        let file = File::from(service_list);
        let mut services = BTreeMap::new();
        services.insert("test-node".to_string(), node);
        let mut volumes = BTreeMap::new();
        volumes.insert("test-chain-data".to_string(), VolumeSetup::default());
        let test_file = File {
            version: "3".to_string(),
            services,
            volumes: Some(volumes),
        };
        assert_eq!(file, test_file);
    }
}
