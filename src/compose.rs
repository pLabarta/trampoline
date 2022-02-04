use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;

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

    pub fn test_module() {
        println!("This is printing from the compose module!");
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Service {
    #[serde(skip)]
    pub name: String,
    pub image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volumes: Option<Vec<Volume>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expose: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ports: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depends_on: Option<Vec<String>>,
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
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct VolumeSetup {}

impl VolumeSetup {
    pub fn default() -> Self {
        VolumeSetup {}
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum VolumeType {
    #[serde(rename(serialize = "volume", deserialize = "volume"))]
    Volume,
    #[serde(rename(serialize = "bind", deserialize = "bind"))]
    Bind,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Volume {
    #[serde(rename(serialize = "type", deserialize = "type"))]
    pub volume_type: VolumeType,
    pub source: String,
    pub target: String,
}
