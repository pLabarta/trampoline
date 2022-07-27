mod service;
pub mod docker;
use service::{Service, ServiceStatus, ServiceKind};
use bollard::{
    container::{CreateContainerOptions, RemoveContainerOptions},
    models::PortBinding,
    network::CreateNetworkOptions,
};
use jsonrpc_core::futures_util::future::{join_all};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::{sync::Arc};

use crate::project::TrampolineProject;


#[derive(Debug, Error)]
pub enum NetworkError {
    #[error(transparent)]
    DockerError(#[from] docker::DockerError),
    #[error(transparent)]
    BollardError(#[from] bollard::errors::Error)
}

type Result<T> = std::result::Result<T, NetworkError>;

#[derive(Serialize, Deserialize, Default)]
pub struct TrampolineNetwork {
    pub name: String,
    pub network: String,
    pub services: Vec<Service>,
}



impl std::fmt::Display for TrampolineNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "New Trampoline development network inititalized.\n
Network name: {}-network\n
Network ID: {}\n
Network Services: {:?}",
            self.name,
            self.id(),
            self.services
                .iter()
                .map(|service| service.name.clone())
                .collect::<Vec<String>>()
        )
    }
}

impl TrampolineNetwork {
    pub async fn new(project: &TrampolineProject, from_config: bool) -> Self {
        match from_config {
            true => {
                // Regenerate network from config file

                // Create default network
                let mut network = TrampolineNetwork::default();

                // Recreate all services from file
                let old_network = TrampolineNetwork::load(project);
                network.name = old_network.name.clone();

                let services = old_network.services.clone();
                old_network.delete().await.expect("Failed to delete current network");
                network.network = create_new_network(project)
                    .await
                    .expect("Failed to create new network");
                // First the nodes so we can get their IP for other services
                let nodes: Vec<&Service> = services
                    .iter()
                    .filter(|&service| matches!(&service.kind, ServiceKind::Ckb))
                    .collect();

                
                let mut ckb_nets = vec![];
                for node in &nodes {
                    let node = node.to_owned();
                    let id = node.id.clone();
                    let fut = TrampolineNetwork::spawn_ckb_service(id, node.clone(), node.ports.clone());
                    let spawn_ckb_net = tokio::spawn(async move{
                        fut.await
                    });
                    ckb_nets.push(spawn_ckb_net);   
                }
                let ckb_services = join_all(ckb_nets)
                    .await.into_iter().map(|s| {
                        match s {
                            Ok(ser) => {
                                ser
                            },
                            Err(_e) => {
                                todo!()
                            }
                        }
                    }).collect::<Vec<_>>();

                network.services.extend(ckb_services);
                // Then indexers
                let indexers: Vec<&Service> = services
                    .iter()
                    .filter(|&service| matches!(&service.kind, ServiceKind::CkbIndexer))
                    .collect();
                let mut idx_nets = vec![];

                for indexer in &indexers {
                    let id = indexer.id.clone();
                    let fut = TrampolineNetwork::spawn_indexer(id, indexer.name.clone(), indexer.ports.clone());
                    let spawn_indexer_net = tokio::spawn(async move {
                        fut.await
                    });
                   idx_nets.push(spawn_indexer_net);
                }
                let indexer_services = join_all(idx_nets).await
                    .into_iter().map(|i| {
                        match i {
                            Ok(ser) => {
                                ser
                            },
                            Err(_e) => {
                                todo!()
                            }
                        }
                    }).collect::<Vec<_>>();
                network.services.extend(indexer_services);

                network
            }
            false => {
                // Create a new default network
                let network_id = create_new_network(project)
                    .await
                    .expect("Failed creating new network");
                Self {
                    name: project.config.name.clone(),
                    services: vec![],
                    network: network_id,
                }
            }
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

    pub async fn status(&self) {
        for service in &self.services {
            let service_status = ServiceStatus::from(service).await;
            println!("{:#?}", service_status);
        }
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

    pub async fn delete(self) -> Result<String> {
        let id = self.id();
        let docker = Arc::new(tokio::sync::Mutex::new(bollard::Docker::connect_with_local_defaults()?));
       
        // Remove all associated containers and their volumes
        let remove_opts = Some(RemoveContainerOptions {
            v: true,
            force: true,
            link: false,
        });
        let mut service_removals = vec![];
        
        for service in self.services {

           
            let service_name = service.name.clone();
            let dock = Arc::clone(&docker);
            let task = tokio::spawn(async move {
                let lock = dock.lock().await;
                lock.remove_container(&service_name, remove_opts).await
             
            });
            service_removals.push(task);
            
        }
        join_all(service_removals).await;
        
        // Remove user defined network
        docker.lock().await
            .remove_network(&format!("{}-network", self.name))
            .await?;
        
        Ok(id)
    }

    pub fn add_service(&mut self, service: &Service) {
        if !self.contains(&service.id) {
            self.services.push(service.clone());
        }
    }

    pub async fn add_indexer(&mut self, node_name: impl AsRef<str>, ports: Vec<(String, String)>) -> Service {
        let indexer_service = TrampolineNetwork::spawn_indexer(self.id(), node_name, ports).await;
        self.add_service(&indexer_service);
        indexer_service
    }


    pub async fn spawn_indexer(id: String, node_name: impl AsRef<str>, ports: Vec<(String, String)>) -> Service {
        let docker = bollard::Docker::connect_with_local_defaults()
        .expect("Failed to connect to Docker API");

        let mut port_bindings = ::std::collections::HashMap::new();

        for port in &ports {
            port_bindings.insert(
                format!("{}/tcp", port.0),
                Some(vec![PortBinding {
                    host_ip: Some(String::from("127.0.0.1")),
                    host_port: Some(port.1.to_string()),
                }]),
            );
        }

        let host_config = Some(bollard::models::HostConfig {
            port_bindings: Some(port_bindings),
            network_mode: Some(id),
            ..Default::default()
        });

        let node_url = format!("http://{}:8114", node_name.as_ref());
        let indexer_config = bollard::container::Config {
            image: Some("nervos/ckb-indexer:latest"),
            host_config,
            cmd: Some(vec!["-s", "data", "-c", &node_url, "-l", "0.0.0.0:8116"]),
            ..Default::default()
        };

        let opts = CreateContainerOptions {
            name: format!("{}-indexer", node_name.as_ref()),
        };

        let indexer_container = docker
            .create_container(Some(opts.clone()), indexer_config)
            .await
            .expect("Failed creating Indexer container");
        let indexer_id = indexer_container.id;

        
        Service {
            name: opts.name,
            id: indexer_id,
            kind: ServiceKind::CkbIndexer,
            ports,
        }
    }

    
    pub async fn spawn_service(id: String, name: impl AsRef<str>, ports: Vec<(String, String)>, kind: ServiceKind) -> Service {
        match kind {
            ServiceKind::Ckb => {
                TrampolineNetwork::spawn_ckb_service(id, name.as_ref(), ports).await
            },
            ServiceKind::CkbIndexer => {
                TrampolineNetwork::spawn_indexer(id, name.as_ref(), ports).await
            }
        }
    }
    pub async fn spawn_ckb_service(id: String, name: impl AsRef<str>, ports: Vec<(String, String)>) -> Service {
        let name = name.as_ref();
        let docker = bollard::Docker::connect_with_local_defaults()
            .expect("Failed to connect to Docker API");

        let mut port_bindings = ::std::collections::HashMap::new();

        for port in &ports {
            port_bindings.insert(
                format!("{}/tcp", port.0),
                Some(vec![PortBinding {
                    host_ip: Some(String::from("127.0.0.1")),
                    host_port: Some(port.1.to_string()),
                }]),
            );
        }

        let host_config = Some(bollard::models::HostConfig {
            port_bindings: Some(port_bindings),
            network_mode: Some(id),
            ..Default::default()
        });

        let ckb_config = bollard::container::Config {
            image: Some("pablitx/ckb-testchain:latest"),
            host_config,
            ..Default::default()
        };

        let opts = CreateContainerOptions { name };

        let ckb_container = docker
            .create_container(Some(opts.clone()), ckb_config)
            .await
            .expect("Failed creating CKB container");
        let ckb_id = ckb_container.id;

        

        
        Service {
            name: opts.name.to_string(),
            id: ckb_id,
            kind: ServiceKind::Ckb,
            ports,
        }
    }

    pub async fn add_ckb(&mut self, name: &str, ports: Vec<(String, String)>) -> Service {
        let docker = bollard::Docker::connect_with_local_defaults()
            .expect("Failed to connect to Docker API");

        let mut port_bindings = ::std::collections::HashMap::new();

        for port in &ports {
            port_bindings.insert(
                format!("{}/tcp", port.0),
                Some(vec![PortBinding {
                    host_ip: Some(String::from("127.0.0.1")),
                    host_port: Some(port.1.to_string()),
                }]),
            );
        }

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

        let opts = CreateContainerOptions { name };

        let ckb_container = docker
            .create_container(Some(opts.clone()), ckb_config)
            .await
            .expect("Failed creating CKB container");
        let ckb_id = ckb_container.id;

        let ckb_service = Service {
            name: opts.name.to_string(),
            id: ckb_id,
            kind: ServiceKind::Ckb,
            ports,
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


pub async fn create_new_network(
    project: &TrampolineProject,
) -> Result<String> {
    let docker =
        bollard::Docker::connect_with_local_defaults().expect("Failed to connect to Docker API");
    let network = CreateNetworkOptions {
        name: format!("{}-network", project.config.name),
        check_duplicate: true,
        ..Default::default()
    };
    let network_respose = docker.create_network(network).await?;
    Ok(network_respose.id.unwrap_or("default-ckb".into()))
}
