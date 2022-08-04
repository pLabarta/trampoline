#![warn(missing_docs)]

//! # Trampoline CLI
//!
//! Welcome to Trampoline CLI documentation!
//!
//! Trampoline CLI is a command-line interface that provides a host of capabilities
//! for running & configuring a local development environment, creating
//! and managing projects, and testing smart contracts.
//! Trampoline currently provides the following features:
//! * Quickly generate new dapp projects
//! * Launch local nodes, miners, and chain indexers
//! * Generate custom schemas for on-chain data
//!
//! ## Installation
//! Currently, Trampoline has only been tested on Linux environments.
//!
//! Trampoline uses docker for running chain services like nodes and miners,
//! so it is recommended to [install docker](https://docs.docker.com/get-docker/) before moving on.
//!
//! To install trampoline crate run: (currently not supported)
//!
//! ```bash
//! cargo install trampoline
//! ```
//! To install from github run:
//!
//! ```bash
//! cargo install --git https://github.com/Tempest-Protocol/trampoline --branch develop
//! ```
//!
//! ## Usage
//! <small>Please note that Trampoline is in early stages and under active development.
//! The API can change drastically.</small>
//!
//! ```bash
//! USAGE:
//!     trampoline <SUBCOMMAND>
//! FLAGS:
//!     -h, --help       Prints help information
//!     -V, --version    Prints version information
//! SUBCOMMANDS:
//!     help       Prints this message or the help of the given subcommand(s)
//!     network    Manage local development chain
//!     new        Create a new Trampoline project
//!     schema     Manage custom on chain structures
//! ```
//!
//! ## Quick Start
//! After installation, use the commands below to create a new trampoline project and manage a local network.
//!  
//! ```bash
//! trampoline new example
//! ```
//! <details>
//! <summary>Click here to see the output.</summary>
//! <br>
//! {{#markdown}}
//! ```bash
//! PATH: generators/Cargo.toml
//! PATH: schemas/src/lib.rs
//! PATH: DockerIndexer
//! PATH: schemas/Cargo.toml
//! PATH: trampoline.toml
//! PATH: generators/src/lib.rs
//! PATH: src/lib.rs
//! PATH: Cargo.toml
//! PATH: Dockerfile.template
//! PATH: trampoline-env.toml
//! [+] Building 0.9s (5/5) FINISHED
//!  => [internal] load build definition from Dockerfile                                                                                                                                                 0.0s
//!  => => transferring dockerfile: 84B                                                                                                                                                                  0.0s
//!  => [internal] load .dockerignore                                                                                                                                                                    0.0s
//!  => => transferring context: 2B                                                                                                                                                                      0.0s
//!  => [internal] load metadata for docker.io/nervos/ckb:v0.103.0                                                                                                                                       0.8s
//!  => CACHED [1/1] FROM docker.io/nervos/ckb:v0.103.0@sha256:a4ca46fee4f19ee383270def46044055dbcca1373eb2e1075eba821d090635cd                                                                          0.0s
//!  => exporting to image                                                                                                                                                                               0.0s
//!  => => exporting layers                                                                                                                                                                              0.0s
//!  => => writing image sha256:5069d16d6a48b181e4a8a9b898ef7cf701c1e16a63c490c0bb0ede12eb01165c                                                                                                         0.0s
//!  => => naming to docker.io/tempest/trampoline-env:latest                                                                                                                                             0.0s
//! Use 'docker scan' to run Snyk tests against images to find vulnerabilities and learn how to fix them
//! ```
//! {{/markdown}}
//! </details>
//!
//! ```bash
//! cd example
//! ```
//!
//! ```bash
//! trampoline network init
//! ```
//!
//! <details>
//! <summary>Click here to see the output.</summary>
//! <br>
//! {{#markdown}}
//! ```bash
//! New Trampoline development network inititalized.
//! Network name: example-network
//! Network ID: 6ad5c643b79d582cc40694541cc9a426bc43c3e9b54265f5bd39668ffdf61494
//! Network Services: ["example-ckb", "example-ckb-indexer"]
//! ```
//! {{/markdown}}
//! </details>
//!
//! ```bash
//! trampoline network launch
//! ```
//!
//! <details>
//! <summary>Click here to see the output.</summary>
//! <br>
//! {{#markdown}}
//! ```bash
//! Nodes should have started!
//! Indexer should have started!
//! ```
//! {{/markdown}}
//! </details>
//!
//! ## Project Layout
//! * `trampoline.toml` - Trampoline project configuration file
//! * `network.toml` - Network services configuration file
//! * `generators` - Transaction generator directory
//! * `schemas` - Custom cell schema directory
//! * `scripts` - Smart contracts directory
//! * `.trampoline` - Directory for caching local chain & indexer data
//!
//! ## Manage Local Network
//! * `trampoline network init` - Initialize a new network
//! * `trampoline network launch` - Launch the network
//! * `trampoline network miner` - Start the miner
//! * `trampoline network indexer` - Start an indexer
//!
//! ## Manage Schemas
//! Schemas use the [molecule](https://github.com/nervosnetwork/molecule) encoding & serialization format.
//!
//! * `trampoline schema new <schema_file>` - Create a new schema
//! * `trampoline schema new byte_10_arr "array my_array [byte;10]"` - Optionally, pass a schema definition (in Molecule)
//! * `trampoline schema build <schema_name>` - Generate rust bindings to build and decode schema
pub mod network;
pub mod opts;
pub mod project;
pub mod schema;
mod utils;
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
pub use network::docker;
use std::path::Path;
use tera::{self, Tera};
pub use utils::*;

include!(concat!(env!("OUT_DIR"), "/templates.rs"));

lazy_static! {
    /// This creates the template files for a trampoline project.
    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        for path in DAPP_FILES.file_names() {
            let name = path
                .strip_prefix("templates/")
                .expect("Failed to remove prefix");
            let content = {
                let file_contents = DAPP_FILES.get(path).expect("read template");
                String::from_utf8(file_contents.to_vec()).expect("template contents")
            };

            tera.add_raw_template(name, &content)
                .expect("failed to add template");
        }
        tera
    };
}

/// Enumeration of a trampoline project and schema.
#[allow(clippy::large_enum_variant)]
pub enum TrampolineResourceType {
    /// Trampoline project type
    Project(project::TrampolineProject),
    /// Schema type
    Schema(schema::Schema),
}

/// Interface for TrampolineResource.
pub trait TrampolineResource {
    type Error;
    type InitArgs;

    /// Load a TrampolineResourceType from a given path.
    fn load(path: impl AsRef<Path>) -> Result<TrampolineResourceType, Self::Error>;

    /// Initialize a TrampolineResourceType from given args.
    fn init(args: Self::InitArgs) -> Result<TrampolineResourceType, Self::Error>;
}

/// Parse a hex string to a vector
pub fn parse_hex(mut input: &str) -> Result<Vec<u8>> {
    if input.starts_with("0x") || input.starts_with("0X") {
        input = &input[2..];
    }
    if input.len() % 2 != 0 {
        return Err(anyhow!("Invalid hex string lenth: {}", input.len()));
    }
    let mut bytes = vec![0u8; input.len() / 2];
    hex_decode(input.as_bytes(), &mut bytes)
        .map_err(|err| anyhow!(format!("parse hex string failed: {:?}", err)))?;
    Ok(bytes)
}
