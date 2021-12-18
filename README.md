# Trampoline Framework
The framework for developing decentralized applications on Nervos Network's Common Knowledge Base.

# Features
Trampoline allows you to jump straight into dapp development without worrying about pesky configuration, installing
a bunch of different tools, etc.

Trampoline projects include all components of a dapp so that you can manage all of the moving parts in one place in
coherent development environment.

- Simple project creation
- Easy account management across environments
- Manage contract deployments and easily generate & decode transactions
- Easily generate custom data structures


# Usage
`trampoline new <project_name>`
`trampoline network new`
`trampoline schema new --name my_schema`
`trampoline build schemas`
`trampoline set miner --address ` This will update the toml in the docker file


Project structure:

env vars:
    - absolute path to base project


- Trampoline.toml
  - name
  - env_mode (docker | virtual)
  - scripts
    - script
      - name
      - size
      - data_hash
      - deployment_lock (script_name || immutable || always_success)
      - deployer address
      - auto_deploy (only in dev chain)
      - arg_schema_name
      - data_schema_name

- Trampoline-env.toml
  - Deployed
    - script
      - outpoint
      - celldep
      - as_script_reference
  - chain
    - type (docker)
    - host
    - port
  - indexer
    - type (docker)
    - host
    - port

schemas/
    schema_name.mol

generators/
    src/lib
    generator_name
        src/bin
        Cargo.toml

Transactions are transaction templates.

Transactions/
transaction-<name>.toml
    -inputs
        - capacity
            - min
            - max
            - auto
        - input
            - capacity
                - min
                - max
                - auto
            - count (any || number || count.min || count.max)
            - lock (default | script_name | script_struct | script_hash)
            - data (none || serialized_value || schema) -- if schema, must pass in paramater for value
            - type (none || script_hash || script_name || script_struct)

    -outputs
        - output
            - count (any || number || count.min || count.max)
            - lock (default | script_name | script_struct | script_hash)
            - data (none || serialized_value || schema) -- if schema, must pass in paramater for value
            - type (none || script_hash || script_name || script_struct)




# Docker commands
start-docker:
	docker build . -t iamm/trampoline-env:latest
	docker run --rm -d -it -p 8114:8114 -p 8115:8115 -p 8116:8116 --name {{PROJ_NAME}} iamm/trampoline-env:latest


stop-docker:
	docker stop {{PROJ_NAME}}

start-ckb:
	docker exec -d -w /ckb/dev {{PROJ_NAME}} ../ckb/ckb run

start-miner:
	docker exec -d -w /ckb/dev {{PROJ_NAME}} ../ckb/ckb miner

start-indexer:
	docker exec -d -w /indexer {{PROJ_NAME}} ./ckb-indexer -l 0.0.0.0:8116 -s ./data


# NEXT STEPS

1. Successfully create each network for trampoline proj by using diff port binding on host machine
   1. Set port binding as env var
2. Fully fledged address manager
3. Docker network
4. ckb-vm-debugger
5. sdk
   1. rpc lib
      1. Use in generators
   2. contract declarations
      1. Use in scripts + generators
   3. Must add base Cargo.toml to templates dir
sdk::network <-- manage a network instance; configs; etc
sdk::address <-- Everything needed for address gen, parsing, etc
sdk::rpc <-- RPC Client
sdk::crypto <-- Crypto operations such as signing, signature verification, hashing
sdk::schema <-- Everything needed to operate on custom schema structures
sdk::contract <-- Contracts are an abstraction over cells & scripts; they have multiple views into them,
                    and can function as middleware for tx generation
sdk::generator <-- State generator library. Exports abstract middleware and base transaction generation logic


Transaction pattern 1: (represents permissioned/admin operation; e.g., allows minting)

  inputs: Input1 where lock_script_hash == SUDT script args

# SUDT Standardization

- nom
- pest
- LALRPOP


def Standard "SUDT" {

  def Parameter "privileged" {
    Is HashedValue::blake2b,
     Sourced Self::Args 
  }

  def Source "Amount" {
    Is Uint(128),
    From Self::Data
  }

  def Rule "OnlyPrivileged" {
    Some(Input | Hash::blake2b(Input::lock) == "privileged")
  }

  def Rule "NoIssuance" {
    ForAll(x in Self::Inputs, y in Self::Outputs | SumOf(x::Amount) == SumOf(y::Amount))
  }

  def Rule "Issuance" {
        ForAll(x in Self::Inputs, y in Self::Outputs | SumOf(x::Amount) < SumOf(y::Amount))
  }

  def Operation "mint" {
    ConstrainedBy(Rules::OnlyPrivileged),
    ConstrainedBy(Rules::Issuance)
  }

  def Operation "transfer" {
    ConstrainedBy(Rules::NoIssuance),
    ConstrainedBy(Rules::OnlyPrivileged)
  }

}

- The set of all valid transaction patterns is the UNION of the patterns described by the Operations
- The set of all Self refers to the set of all cells that are standard compliant AND have the same parameters
- Parameters are values read from the environment that are constant across a type; i.e., two elements of Self are the same
  - only if their parameters are the same and they share the same standard
- Parameters do not change across transactions while Sources can



Keywords:

"def"

"Standard"

"Parameter"

"in"

"Some"

"All"

"ForAll"

"Is"

"|"

"Self"

"Rule"
