use std::prelude::v1::*;

pub mod mol_defs;
use mol_defs::{NFT, NFTBuilder, NFTReader, Byte32, Byte32Reader};
use crate::ckb_types::{
    bytes::Bytes,
    packed::{self, Uint128},
    prelude::*,
};


use crate::{
    contract::{schema::SchemaPrimitiveType}, impl_entity_unpack, impl_pack_for_primitive, impl_primitive_reader_unpack, impl_pack_for_fixed_byte_array,
};
#[cfg(not(feature = "script"))]
use crate::contract::Contract;
#[cfg(not(feature = "script"))]
use ckb_jsonrpc_types::{Byte32 as JsonByte32, Uint128 as JsonUint128};
#[cfg(not(feature = "script"))]
use ckb_hash::{new_blake2b, blake2b_256};
#[cfg(not(feature = "script"))]
pub trait NftContentHasher {
    fn hash(content: impl AsRef<[u8]>) -> mol_defs::Byte32;
}


impl_pack_for_fixed_byte_array!([u8; 32], Byte32);
impl_primitive_reader_unpack!([u8;32], Byte32Reader, 32, from);
impl_entity_unpack!([u8;32], Byte32);

pub type GenesisId = SchemaPrimitiveType<[u8;32], Byte32>;
pub type ContentId = SchemaPrimitiveType<[u8;32], Byte32>;

#[derive(Debug, Clone, Default)]
pub struct TrampolineNFT {
    pub genesis_id: GenesisId,
    pub cid: ContentId,
}

#[cfg(not(feature = "script"))]
pub type TrampolineNFTContract = Contract<(), TrampolineNFT>;



