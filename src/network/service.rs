use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum ServiceKind {
    Ckb,
    CkbIndexer,
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Service {
   pub name: String,
   pub id: String,
   pub kind: ServiceKind,
   pub ports: Vec<(String, String)>,
}

impl AsRef<str> for Service {
    fn as_ref(&self) -> &str {
        self.name.as_str()
    }
}

#[derive(Debug)]
pub struct ServiceStatus {
    service: Service,
    running: bool,
    created: bool,
    up_time: Option<String>,
    ports: String,
}

impl ServiceStatus {
    pub async fn from(service: &Service) -> Self {
        let docker = bollard::Docker::connect_with_local_defaults()
            .expect("Failed to connect to Docker API");
        let container_info = docker
            .inspect_container(&service.name.to_string(), None)
            .await
            .unwrap();
        let ports = {
            let map = &container_info
                .clone()
                .network_settings
                .unwrap()
                .ports
                .unwrap();
            let mut ports_string = String::new();

            for (container_port, bindings) in map {
                match bindings {
                    Some(binding) => {
                        for b in binding {
                            let host_port = b.host_port.as_ref().unwrap();
                            let formatted_ports = format!("{}:{}", container_port, host_port);
                            ports_string.push_str(&formatted_ports);
                        }
                    }
                    None => {}
                }
            }

            ports_string
        };

        Self {
            service: service.clone(),
            running: container_info.clone().state.unwrap().running.unwrap(),
            created: true,
            up_time: Some(format!(
                "Started at: {}",
                container_info.state.unwrap().started_at.unwrap()
            )),
            ports,
        }
    }
}
