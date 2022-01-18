pub mod mol_defs;

use std::marker::PhantomData;

use ckb_types::{
    bytes::Bytes,
    packed::{self},
    prelude::*,
};

use ckb_jsonrpc_types::{JsonBytes, Uint32 as JsonUint32};
use crate::{contract::*, impl_pack_for_primitive, impl_primitive_reader_unpack, impl_entity_unpack};
use mol_defs::{Uint8, Uint8Reader, Issuer, Uint16, Uint16Reader, Uint32, Uint32Reader, Info};



// Transform to molecule entities
impl_pack_for_primitive!(u8, Uint8);
impl_pack_for_primitive!(u16, Uint16);
impl_pack_for_primitive!(u32, Uint32);

// Transform from molecule entity readers
impl_primitive_reader_unpack!(u8, Uint8Reader, 1, from_le_bytes);
impl_primitive_reader_unpack!(u16, Uint16Reader, 2, from_le_bytes);
impl_primitive_reader_unpack!(u32, Uint32Reader, 4, from_le_bytes);

// Transform from molecule entities
impl_entity_unpack!(u8, Uint8);
impl_entity_unpack!(u16, Uint16);
impl_entity_unpack!(u32, Uint32);



pub type Version = SchemaPrimitiveType<u8, Uint8>;

pub type ClassCount = SchemaPrimitiveType<u32, Uint32>;

pub type SetCount = SchemaPrimitiveType<u32, Uint32>;

pub type InfoSize = SchemaPrimitiveType<u16, Uint16>;


pub struct NftInfo(Vec<Bytes>);

// Serializes to two different entities that are then packed together:
// Fixed size data (version, class count, set count, info size)
// Dynamic size data (info)
#[derive(Debug, Clone, Default)]
pub struct NftIssuer {
    pub version: Version,
    pub class_count: ClassCount,
    pub set_count: SetCount,
    pub info_size: InfoSize,
    pub info: Bytes,
}



#[test]
fn test_mol_conversion_schema_prim_u32() {
    let primitive_val = SchemaPrimitiveType::<u32, Uint32>::new(5);

    let comparison_prim = SchemaPrimitiveType::<u32, Uint32>::from_mol(5_u32.pack());
    assert_eq!(primitive_val.to_mol().unpack(), comparison_prim.to_mol().unpack());
}

