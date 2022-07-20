use std::future::Future;

use bollard::{
    container::CreateContainerOptions, models::PortBinding, network::CreateNetworkOptions,
};
use jsonrpc_core::futures_util::future::join_all;
use serde::{Deserialize, Serialize};

use crate::project::TrampolineProject;

#[derive(Serialize, Deserialize, Clone)]
pub enum ServiceKind {
    Ckb,
    CkbIndexer,
}

#[derive(Serialize, Deserialize)]
pub struct TrampolineNetwork {
    pub name: String,
    pub network: String,
    pub services: Vec<Service>,
}

impl TrampolineNetwork {
    pub async fn new(project: &TrampolineProject) -> Self {
        let network_id = create_new_network(project)
            .await
            .expect("Failed creating new network");
        Self {
            name: project.config.name.clone(),
            services: vec![],
            network: network_id,
        }
    }

    pub async fn run(&self) {
        let docker = bollard::Docker::connect_with_local_defaults()
            .expect("Failed to connect to Docker API");

        // First run nodes
        let nodes: Vec<&Service> = self
            .services
            .iter()
            .filter(|&service| matches!(&service.kind, ServiceKind::Ckb))
            .collect();
        let mut starting_nodes = Vec::with_capacity(nodes.len());
        for node in nodes {
            starting_nodes.push(docker.start_container::<String>(&node.id, None));
        }
        join_all(starting_nodes).await;
        println!("Nodes should have started!");

        // Then run indexers
        let indexers: Vec<&Service> = self
            .services
            .iter()
            .filter(|&service| matches!(&service.kind, ServiceKind::CkbIndexer))
            .collect();
        let mut starting_indexers = Vec::with_capacity(indexers.len());
        for indexer in indexers {
            starting_indexers.push(docker.start_container::<String>(&indexer.id, None));
        }
        join_all(starting_indexers).await;
        println!("Indexer should have started!");
    }

    pub async fn stop(&self) {
        let docker = bollard::Docker::connect_with_local_defaults()
            .expect("Failed to connect to Docker API");
        let mut stopping_services = Vec::with_capacity(self.services.len());
        for service in &self.services {
            stopping_services.push(docker.stop_container(&service.name, None));
        }
        join_all(stopping_services).await;
        println!("Trampoline Network stopped")
    }

    pub async fn reset(&self, service_name: String) {
        let service = self
            .services
            .iter()
            .find(|service| service.name == service_name);
        match service {
            None => {
                println!("Service not found in current network config")
            }
            Some(service) => {
                let docker = bollard::Docker::connect_with_local_defaults()
                    .expect("Failed to connect to Docker API");
                docker
                    .stop_container(&service.name, None)
                    .await
                    .unwrap_or_else(|_| panic!("Failed to stop {}", &service.name));
                docker
                    .start_container::<String>(&service.id, None)
                    .await
                    .unwrap_or_else(|_| panic!("Failed to start {}", &service.name));
                println!("Service {} restarted", &service.name);
            }
        }
    }

    pub fn load(project: &TrampolineProject) -> Self {
        let path = project.root_dir.join("network.toml");
        let toml = std::fs::read_to_string(path).expect("Unable to read network config from file");
        toml::from_str(&toml).expect("Failed parsing network config file")
    }

    pub fn save(&self, project: &TrampolineProject) {
        let path = project.root_dir.join("network.toml");
        let toml = toml::to_string(self).expect("Failed converting network config into TOML");
        std::fs::write(path, toml).expect("Failed to write network config to file");
    }

    pub fn add_service(&mut self, service: &Service) {
        if !self.contains(&service.id) {
            self.services.push(service.clone());
        }
    }

    pub async fn add_indexer(&mut self, node: &Service) -> Service {
        let docker = bollard::Docker::connect_with_local_defaults()
            .expect("Failed to connect to Docker API");

        let mut port_bindings = ::std::collections::HashMap::new();
        port_bindings.insert(
            String::from("8116/tcp"),
            Some(vec![PortBinding {
                host_ip: Some(String::from("127.0.0.1")),
                host_port: Some(String::from("8116")),
            }]),
        );

        let host_config = Some(bollard::models::HostConfig {
            port_bindings: Some(port_bindings),
            network_mode: Some(self.id()),
            ..Default::default()
        });

        let node_url = format!("http://{}:8114", node.name);
        let indexer_config = bollard::container::Config {
            image: Some("nervos/ckb-indexer:latest"),
            host_config,
            cmd: Some(vec!["-s", "data", "-c", &node_url, "-l", "0.0.0.0:8116"]),
            ..Default::default()
        };

        let opts = CreateContainerOptions {
            name: format!("{}-ckb-indexer", self.name),
        };

        let indexer_container = docker
            .create_container(Some(opts.clone()), indexer_config)
            .await
            .expect("Failed creating Indexer container");
        let indexer_id = indexer_container.id;

        let indexer_service = Service {
            name: opts.name,
            id: indexer_id,
            kind: ServiceKind::CkbIndexer,
        };

        self.add_service(&indexer_service);
        indexer_service
    }

    pub async fn add_ckb(&mut self) -> Service {
        let docker = bollard::Docker::connect_with_local_defaults()
            .expect("Failed to connect to Docker API");

        let mut port_bindings = ::std::collections::HashMap::new();
        port_bindings.insert(
            String::from("8114/tcp"),
            Some(vec![PortBinding {
                host_ip: Some(String::from("127.0.0.1")),
                host_port: Some(String::from("8114")),
            }]),
        );

        let host_config = Some(bollard::models::HostConfig {
            port_bindings: Some(port_bindings),
            network_mode: Some(self.id()),
            ..Default::default()
        });

        let ckb_config = bollard::container::Config {
            image: Some("pablitx/ckb-testchain:latest"),
            host_config,
            ..Default::default()
        };

        let opts = CreateContainerOptions {
            name: format!("{}-ckb-node", self.name),
        };

        let ckb_container = docker
            .create_container(Some(opts.clone()), ckb_config)
            .await
            .expect("Failed creating CKB container");
        let ckb_id = ckb_container.id;

        let ckb_service = Service {
            name: opts.name,
            id: ckb_id,
            kind: ServiceKind::Ckb,
        };

        self.add_service(&ckb_service);
        ckb_service
    }

    pub fn contains(&self, container_id: &String) -> bool {
        for service in &self.services {
            if &service.id == container_id {
                return true;
            }
        }
        false
    }

    pub fn get_service(&self, service_name: String) -> Option<&Service> {
        self.services
            .iter()
            .find(|service| service.name == service_name)
    }

    pub fn id(&self) -> String {
        self.network.clone()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Service {
    pub name: String,
    pub id: String,
    pub kind: ServiceKind,
}

pub async fn create_new_network(
    project: &TrampolineProject,
) -> Result<String, bollard::errors::Error> {
    let docker =
        bollard::Docker::connect_with_local_defaults().expect("Failed to connect to Docker API");
    let network = CreateNetworkOptions {
        name: format!("{}-network", project.config.name),
        check_duplicate: true,
        ..Default::default()
    };
    let network_respose = docker.create_network(network).await?;
    Ok(network_respose.id.unwrap())
}
