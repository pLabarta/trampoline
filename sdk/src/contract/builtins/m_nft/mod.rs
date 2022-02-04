pub mod mol_defs;

use std::marker::PhantomData;

use ckb_hash::new_blake2b;
use ckb_types::{
    bytes::Bytes,
    packed::{self, CellInput},
    prelude::*,
};

use ckb_jsonrpc_types::{JsonBytes, Uint32 as JsonUint32};
use crate::{contract::*, impl_pack_for_primitive, impl_primitive_reader_unpack, impl_entity_unpack};
use mol_defs::{Uint8, Uint8Reader, Issuer, Uint16, Uint16Reader, Uint32, Uint32Reader, Info, TypeId};



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

pub type IssuerArgs = SchemaPrimitiveType<[u8; 20], TypeId>;

pub struct NftInfo<T: AsRef<Bytes>>(Vec<T>);

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

#[derive(Debug, Clone, Default)]
pub struct NftIssuerArgs(pub IssuerArgs);

impl NftIssuerArgs {
    pub fn from_cell_input(input: &CellInput, idx: u64) -> Self {
        let mut hashed_input = [0u8;32];
        let mut hasher = new_blake2b();
        hasher.update(input.as_slice());
        hasher.update(&idx.to_le_bytes());
        hasher.finalize(&mut hashed_input);
        let mut type_id = [0u8;20];
        type_id.copy_from_slice(&hashed_input[..20]);
        Self(IssuerArgs::new(type_id))
    }
}

impl BytesConversion for NftIssuer {
    fn from_bytes(bytes: Bytes) -> Self {
        todo!()
    }

    fn to_bytes(&self) -> Bytes {
        todo!()
    }
}

impl MolConversion for NftIssuer {
    type MolType = Issuer;

    fn to_mol(&self) -> Self::MolType {
        todo!()
    }

    fn from_mol(entity: Self::MolType) -> Self {
        todo!()
    }
}

impl JsonByteConversion for NftIssuer {
    fn to_json_bytes(&self) -> JsonBytes {
        todo!()
    }

    fn from_json_bytes(bytes: JsonBytes) -> Self {
        todo!()
    }
}

impl JsonConversion for NftIssuer {
    type JsonType = ckb_jsonrpc_types;

    fn to_json(&self) -> Self::JsonType {
        todo!()
    }

    fn from_json(json: Self::JsonType) -> Self {
        todo!()
    }
}


#[test]
fn test_mol_conversion_schema_prim_u32() {
    let primitive_val = SchemaPrimitiveType::<u32, Uint32>::new(5);

    let comparison_prim = SchemaPrimitiveType::<u32, Uint32>::from_mol(5_u32.pack());
    assert_eq!(primitive_val.to_mol().unpack(), comparison_prim.to_mol().unpack());
}

