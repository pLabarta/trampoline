//! Definitions for command line interface

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    about = "Create, configure, build, and deploy fullstack dapps running on Nervos Network",
    name = "trampoline",
    alias = "tram"
)]

/// Enumeration of trampoline commands.
pub enum TrampolineCommand {
    /// Create a new Trampoline project.
    #[structopt(name = "new", alias = "n")]
    #[structopt(about = "Create a new Trampoline project")]
    NewProject {
        /// Project name
        name: String,
    },

    /// Network commands for managing local development chain.
    #[structopt(
        name = "network",
        alias = "net",
        about = "Manage local development chain"
    )]
    Network {
        /// Network command type.
        #[structopt(flatten)]
        command: NetworkCommands,
    },

    /// Manage custom on chain structures.
    #[structopt(
        name = "schema",
        about = "Manage custom on chain structures",
        alias = "s"
    )]
    Schema {
        /// Schema command type.
        #[structopt(flatten)]
        command: SchemaCommand,
    },

    /// Check a local trampoline project for issues and repair them.
    #[structopt(
        name = "check",
        about = "Check a local trampoline project for issues & repair them"
    )]
    Check,
    // #[structopt(name = "account", about = "Manage addresses and keys")]
    // Account {
    //     #[structopt(flatten)]
    //     command: AccountCommand
    // }
}

/// Manage local development network.
#[derive(Debug, StructOpt)]
pub enum NetworkCommands {
    /// Initialize a new network configuration without starting.
    #[structopt(
        name = "init",
        about = "Initialize new network configuration without starting",
        alias = "i"
    )]
    Init {},

    /// Initialize a new network from `network.toml` file.
    #[structopt(
        name = "recreate",
        about = "Initialize new network from network.toml file"
    )]
    Recreate {},

    /// Launch a local development network.
    #[structopt(
        name = "launch",
        about = "Launch local development network",
        alias = "l"
    )]
    Launch {},

    /// Stop a local development network.
    #[structopt(name = "stop", about = "Stop local development network", alias = "s")]
    Stop {},

    /// Reset a local development network.
    #[structopt(name = "reset", about = "Reset local development network", alias = "r")]
    Reset {
        /// Name of the service to reset.
        service: Option<String>,
    },

    /// Show logs for a given network service.
    #[structopt(name = "logs", about = "Show logs for a particular network service")]
    Logs {
        /// Service name.
        service: String,
        /// Log output.
        #[structopt(short, long)]
        output: Option<PathBuf>,
    },

    /// Print the status of a local development network.
    #[structopt(name = "status", about = "Print status of local development network")]
    Status {},

    /// Remove docker containers and local development network from system.
    #[structopt(
        name = "delete",
        about = "Remove local development containers and network from system"
    )]
    Delete {},

    /// Set the miner address so blocks can be mined locally.
    #[structopt(
        name = "set-miner",
        about = "Set the miner address so blocks can be mined locally"
    )]
    SetMiner {
        /// Public key
        #[structopt(name = "pubkey", required_unless = "lock_arg", long)]
        pubkey: Option<String>,
        /// CKB `lock_arg`
        #[structopt(name = "lock_arg", required_unless = "pubkey", long)]
        lock_arg: Option<String>,
    },

    /// Configure your local developer network. You can also manually edit `trampoline-env.toml`.
    #[structopt(
        name = "config",
        alias = "c",
        about = "Configure your local developer network. You can also manually edit `trampoline-env.toml`"
    )]
    Config {
        /// Host port
        #[structopt(name = "host-port", long, short)]
        port_host: Option<usize>,
        /// Host name
        #[structopt(name = "host", long, short)]
        host: Option<String>,
        /// Local path binding.
        #[structopt(name = "local-path-binding", long, short)]
        local_binding: Option<PathBuf>,
    },

    /// Launch the indexer for improved queries.
    #[structopt(name = "index", about = "Launch the indexer for improved queries")]
    Indexer {},

    /// Start continuously mining blocks or mine a single block.
    #[structopt(
        name = "miner",
        about = "Start continuously mining blocks or mine a single block"
    )]
    Miner {
        /// Whether to mine one block or more.
        one_block: Option<bool>,
    },

    /// Make RPC calls.
    #[structopt(name = "rpc", about = "Make Rpc calls")]
    Rpc {
        /// Hash string.
        hash: String,
    },
    #[structopt(
        name = "launchold",
        about = "Launch local development network",
        alias = "lo"
    )]
    LaunchOld {},
}

/// Possible schema commands.
#[derive(Debug, StructOpt)]
pub enum SchemaCommand {
    /// Inititalize a new schema.
    #[structopt(name = "new", about = "Initialize a new schema")]
    New {
        /// Schema name.
        name: String,
        /// Schema definition.
        def: Option<String>,
    },
    /// Generate Rust bindings for a schema.
    #[structopt(name = "build", about = "Generate rust bindings for schema")]
    Build {
        /// Schema name.
        name: String,
    },
}
