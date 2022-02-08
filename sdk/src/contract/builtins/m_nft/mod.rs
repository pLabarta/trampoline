pub mod mol_defs;

use std::marker::PhantomData;

use ckb_hash::new_blake2b;
use ckb_types::{
    bytes::Bytes,
    packed::{self, CellInput},
    prelude::*,
};

use crate::contract::builtins::m_nft::mol_defs::IssuerBuilder;
use crate::{
    contract::*, impl_entity_unpack, impl_pack_for_primitive, impl_primitive_reader_unpack,
};
use ckb_jsonrpc_types::{JsonBytes, Uint32 as JsonUint32};
use mol_defs::{
    Info, Issuer, TypeId, Uint16, Uint16Reader, Uint32, Uint32Reader, Uint8, Uint8Reader,
};

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

pub type mNFTIssuer = SchemaPrimitiveType<NftIssuer, Issuer>;

pub struct NftInfo<T: AsRef<Bytes>>(Vec<T>);

// Serializes to two different entities that are then packed together:
// Fixed size data (version, class count, set count, info size)
// Dynamic size data (info)
// For types that do not map directly to a builtin ckb type, Pack and Unpack must be implemented as well (if we want it to be SchemaPrimitiveType)
// The benefit of SchemaPrimitiveType<T,M> is that we get all of the other traits automatically
// NOTE: Currently we ignore NftIssuer.info and keep info_size at 0
#[derive(Debug, Clone, Default)]
pub struct NftIssuer {
    pub version: Version,
    pub class_count: ClassCount,
    pub set_count: SetCount,
    pub info_size: InfoSize,
    pub info: Bytes,
}

impl Pack<Issuer> for NftIssuer {
    fn pack(&self) -> Issuer {
        IssuerBuilder::default()
            .class_count(self.class_count.to_mol())
            .info_size(self.info_size.to_mol())
            .set_count(self.set_count.to_mol())
            .version(self.version.to_mol())
            .build()
    }
}

impl Unpack<NftIssuer> for Issuer {
    fn unpack(&self) -> NftIssuer {
        let reader = self.as_reader();
        NftIssuer {
            version: Version::from_mol(reader.version().to_entity()),
            class_count: ClassCount::from_mol(reader.class_count().to_entity()),
            set_count: SetCount::from_mol(reader.set_count().to_entity()),
            info_size: InfoSize::from_mol(reader.info_size().to_entity()),
            // We simply do not support info right now since the encoding of Issuer data in mNFT standard
            // is not a molecule Table, and NftIssuer is meant to encapsulate the entire Data field of an Issuer cell
            info: Default::default(),
        }
    }
}

impl Pack<TypeId> for [u8; 20] {
    fn pack(&self) -> TypeId {
        TypeId::from_slice(&self[..]).expect("unable to create TypeId from slice")
    }
}

impl Unpack<[u8; 20]> for TypeId {
    fn unpack(&self) -> [u8; 20] {
        let ptr = self.as_slice().as_ptr() as *const [u8; 20];
        unsafe { *ptr }
    }
}

// NftIssuerArgs is used for TypeID creation
// To do: for newtypes OVER schema primitive types, provide macro
// for redirecting calls to *Conversion provided methods to the inner type
#[derive(Debug, Clone, Default)]
pub struct NftIssuerArgs(pub IssuerArgs);

impl NftIssuerArgs {
    pub fn from_cell_input(input: &CellInput, idx: u64) -> Self {
        let mut hashed_input = [0u8; 32];
        let mut hasher = new_blake2b();
        hasher.update(input.as_slice());
        hasher.update(&idx.to_le_bytes());
        hasher.finalize(&mut hashed_input);
        let mut type_id = [0u8; 20];
        type_id.copy_from_slice(&hashed_input[..20]);
        Self(IssuerArgs::new(type_id))
    }
}

impl BytesConversion for NftIssuerArgs {
    fn from_bytes(bytes: Bytes) -> Self {
        Self(IssuerArgs::from_bytes(bytes))
    }

    fn to_bytes(&self) -> Bytes {
        self.0.to_bytes()
    }
}

impl MolConversion for NftIssuerArgs {
    type MolType = TypeId;

    fn to_mol(&self) -> Self::MolType {
        self.0.to_mol()
    }

    fn from_mol(entity: Self::MolType) -> Self {
        Self(IssuerArgs::from_mol(entity))
    }
}

impl JsonConversion for NftIssuerArgs {
    type JsonType = JsonBytes;

    fn to_json(&self) -> Self::JsonType {
        self.0.to_json()
    }

    fn from_json(json: Self::JsonType) -> Self {
        Self(IssuerArgs::from_json(json))
    }
}

impl JsonByteConversion for NftIssuerArgs {
    fn to_json_bytes(&self) -> JsonBytes {
        self.0.to_json_bytes()
    }

    fn from_json_bytes(bytes: JsonBytes) -> Self {
        Self(IssuerArgs::from_json_bytes(bytes))
    }
}

pub type MultiNFTIssuerContract = Contract<NftIssuerArgs, mNFTIssuer>;

#[test]
fn test_mol_conversion_schema_prim_u32() {
    let primitive_val = SchemaPrimitiveType::<u32, Uint32>::new(5);

    let comparison_prim = SchemaPrimitiveType::<u32, Uint32>::from_mol(5_u32.pack());
    assert_eq!(
        primitive_val.to_mol().unpack(),
        comparison_prim.to_mol().unpack()
    );
}
