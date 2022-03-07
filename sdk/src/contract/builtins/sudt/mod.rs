use std::prelude::v1::*;

use crate::ckb_types::{
    packed::{Byte32, Uint128},
};
#[cfg(not(feature = "script"))]
use crate::contract::Contract;


use crate::contract::schema::SchemaPrimitiveType;

#[derive(Debug, Clone, Default)]
struct InnerOwnerLockHash([u8; 32]);

#[derive(Debug, Clone, Default)]
struct InnerSudtAmount(u128);

pub type OwnerLockHash = SchemaPrimitiveType<[u8; 32], Byte32>;
pub type SudtAmount = SchemaPrimitiveType<u128, Uint128>;

#[cfg(not(feature = "script"))]
pub type SudtContract = Contract<OwnerLockHash, SudtAmount>;
