use std::prelude::v1::*;

use crate::ckb_types::{
    bytes::Bytes,
    packed::{self, Byte32, Uint128},
    prelude::*,
};
#[cfg(not(feature = "script"))]
use crate::contract::Contract;
#[cfg(not(feature = "script"))]
use ckb_jsonrpc_types::{Byte32 as JsonByte32, Uint128 as JsonUint128};

use crate::contract::schema::SchemaPrimitiveType;

#[derive(Debug, Clone, Default)]
struct InnerOwnerLockHash([u8; 32]);

#[derive(Debug, Clone, Default)]
struct InnerSudtAmount(u128);

pub type OwnerLockHash = SchemaPrimitiveType<[u8; 32], Byte32>;
pub type SudtAmount = SchemaPrimitiveType<u128, Uint128>;

#[cfg(not(feature = "script"))]
pub type SudtContract = Contract<OwnerLockHash, SudtAmount>;
