# Trampoline Framework
The framework for developing decentralized applications on Nervos Network's Common Knowledge Base.

# Features
Trampoline allows you to jump straight into dapp development without worrying about pesky configuration, installing
a bunch of different tools, etc.

Trampoline projects include all components of a dapp so that you can manage all of the moving parts in one place in
coherent development environment.

- [x] Quickly generate new trampoline projects.
- [x] Start and stop local dev chain with ease.
- [x] Start and stop local indexer(s) with ease.
- [x] Add your own miner(s).
- [x] Autogenerate Rust bindings for custom schemas for use on and off chain.
- []  Manage accounts and addresses across developer, staging, and deployment environments.
- []  Indexer extensions to index custom schemas.
- []  Trampoline server API powered by Rocket-rs for transaction generation & querying.
- []  Declaratively define transaction patterns for easy transaction creation.
- []  Compile transaction patterns to CKB scripts 


# Installation
Currently, just ensure the Rust toolchain is installed on your machine. Then, install with Cargo:

`cargo install trampoline --git https://github.com/WilfredTA/trampoline`

You also need to have Docker installed.

# Usage

## Start a new project
`trampoline new <project_name>`

## Manage local network
To initialize the network: `trampoline network new`

To start the node: `trampoline network launch`

To set a miner: `trampoline network set-miner [lock_arg | pubkey]`

To start the miner: `trampoline network miner`

To start an indexer: `trampoline network indexer`

## Manage schemas

Create a new schema: `trampoline schema new <schema_name>`

Optionally, schema definition (in Molecule) can be passed in: `trampoline schema new byte_10_arr "array my_array [byte; 10]"`

Generate rust bindings to build and decode schema: `trampoline schema build <schema_name>`

