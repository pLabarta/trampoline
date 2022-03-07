use std::prelude::v1::*;

pub mod mol_defs;
use crate::ckb_types::{
    bytes::Bytes,
};
use mol_defs::{Byte32, Byte32Reader};

#[cfg(not(feature = "script"))]
use crate::contract::Contract;
use crate::{
    contract::schema::SchemaPrimitiveType, impl_entity_unpack, impl_pack_for_fixed_byte_array, impl_primitive_reader_unpack,
};


#[cfg(not(feature = "script"))]
pub trait NftContentHasher {
    fn hash(content: impl AsRef<[u8]>) -> mol_defs::Byte32;
}

impl_pack_for_fixed_byte_array!([u8; 32], Byte32);
impl_primitive_reader_unpack!([u8; 32], Byte32Reader, 32, from);
impl_entity_unpack!([u8; 32], Byte32);

pub type GenesisId = SchemaPrimitiveType<[u8; 32], Byte32>;
pub type ContentId = SchemaPrimitiveType<[u8; 32], Byte32>;

#[derive(Debug, Clone, Default)]
pub struct TrampolineNFT {
    pub genesis_id: GenesisId,
    pub cid: ContentId,
}

#[cfg(not(feature = "script"))]
pub type TrampolineNFTContract = Contract<(), TrampolineNFT>;
