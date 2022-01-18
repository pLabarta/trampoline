use ckb_types::{
    bytes::Bytes,
    packed::{self, Byte32, Uint128},
    prelude::*,
};

use ckb_jsonrpc_types::{Byte32 as JsonByte32, Uint128 as JsonUint128};
use crate::contract::*;

// To do: Some of these should be try_from
// TO DO: Implement From<u128>, From<JsonBytes>, From<Bytes>, From<Uint128>, From<JsonUint128> for SudtAmount
// TO DO: Implement From<[u8;32]>, From<JsonBytes>, From<Bytes>, From<Byte32>, From<JsonByte32> 
// TO DO: TryFrom and AsRef
// TO DO: ckb script (no-std) compatible implementation as well.


// Situations
// Newtype with inner type that implements Pack<Entity> and for which the entity implements Unpack<inner_type>
// Compound type where various fields are new_types as the above, and the compound type's fields are equivalent to the attributes of a 
// Molecule-generated table
#[derive(Debug, Clone, Default)]
pub struct OwnerLockHash([u8;32]);

#[derive(Debug, Clone, Default)]
pub struct SudtAmount(u128);

impl BytesConversion for OwnerLockHash {
    fn from_bytes(bytes: Bytes) -> Self {
        Self(Byte32::from_compatible_slice(bytes.as_ref()).expect("Unable to build Byte32 from bytes").unpack())
    }

    fn to_bytes(&self) -> Bytes {
        self.to_mol().as_bytes()
    }
}

impl JsonByteConversion for OwnerLockHash {
    fn to_json_bytes(&self) -> JsonBytes {
        self.to_mol().as_bytes().pack().into()
    }

    fn from_json_bytes(bytes: JsonBytes) -> Self {
        Self::from_bytes(bytes.into_bytes())
    }
}

impl JsonConversion for OwnerLockHash {
    type JsonType = JsonByte32;

    fn to_json(&self) -> Self::JsonType {
        self.to_mol().into()
    }

    fn from_json(json: Self::JsonType) -> Self {
        OwnerLockHash(packed::Byte32::from(json).unpack())
    }
}

impl MolConversion for OwnerLockHash {
    type MolType = Byte32;

    fn to_mol(&self) -> Self::MolType {
        self.0.pack()
    }

    fn from_mol(entity: Self::MolType) -> Self {
        Self(entity.unpack())
    }
}


impl BytesConversion for SudtAmount {
    fn from_bytes(bytes: Bytes) -> Self {
        Self(u128::from_bytes(bytes))
    }

    fn to_bytes(&self) -> Bytes {
        self.to_mol().as_bytes()
    }
}

impl JsonByteConversion for SudtAmount {
    fn to_json_bytes(&self) -> JsonBytes {
        self.to_mol().as_bytes().pack().into()
    }

    fn from_json_bytes(bytes: JsonBytes) -> Self {
        Self::from_bytes(bytes.into_bytes())
    }
}

impl JsonConversion for SudtAmount {
    type JsonType = JsonUint128;

    fn to_json(&self) -> Self::JsonType {
        Self::JsonType::from(self.0)
    }

    fn from_json(json: Self::JsonType) -> Self {
        Self(json.into())
    }
}

impl MolConversion for SudtAmount {
    type MolType = Uint128;

    fn to_mol(&self) -> Self::MolType {
        self.0.pack()
    }

    fn from_mol(entity: Self::MolType) -> Self {
        Self(entity.unpack())
    }
}

pub trait PrimitiveTypeFromBytes: Sized {
    type Entity: Entity + Unpack<Self>;
    fn from_bytes(bytes: Bytes) -> Self {
        Self::Entity::from_compatible_slice(bytes.as_ref())
            .expect("Unable to generate value from bytes")
            .unpack()
    }
}

impl PrimitiveTypeFromBytes for u128 {
    type Entity = Uint128;
}

impl From<u128> for SudtAmount {
    fn from(n: u128) -> Self {
        Self(n)
    }
}

impl From<SudtAmount> for u128 {
    fn from(n: SudtAmount) -> Self {
        n.0
    }
}
impl From<[u8;32]> for OwnerLockHash {
    fn from(s: [u8; 32]) -> Self {
        Self(s)
    }
}

// impl SudtAmount {
//     pub fn to_mol(&self) -> Uint128 {
//         self.0.pack()
//     }
//     fn from_mol(entity: Uint128) -> Self {
//         Self(entity.unpack())
//     }

//     fn from_bytes(bytes: Bytes) -> Self {
//         Self(u128::from_bytes(bytes))
//     }

//     pub fn to_bytes(&self) -> Bytes {
//         self.to_mol().as_bytes()
//     }
// }

// // jsonrpc compatible conversions

// impl OwnerLockHash {

//     // Self::PackedType needs to impl Into<JsonType> 
//     pub fn to_json(&self) -> JsonByte32 {
//         self.to_mol().into()
//     }

//     // Arg needs to impl Into<Self::PackedType>
//     pub fn from_json(json_byte32: JsonByte32) -> Self {
//         OwnerLockHash(packed::Byte32::from(json_byte32).unpack())
//     }

//     pub fn to_json_bytes(&self) -> ckb_jsonrpc_types::JsonBytes {
//         self.to_mol().as_bytes().pack().into()
//     }

//     pub fn from_json_bytes(bytes: JsonBytes) -> Self {
//         Self::from_bytes(bytes.into_bytes())
//     }
// }

// // Ckb script-compatible conversions
// impl OwnerLockHash {
//     pub fn to_mol(&self) -> Byte32 {
//         self.0.pack()
//     }

//     // Arg needs to impl Unpack<Self::InnerType>
//     pub fn from_mol(entity: Byte32) -> Self {
//         Self(entity.unpack())
//     }

//     pub fn from_bytes(bytes: Bytes) -> Self {
//         Self(Byte32::from_compatible_slice(bytes.as_ref()).expect("Unable to build Byte32 from bytes").unpack())
//     }

//     pub fn to_bytes(&self) -> Bytes {
//         self.to_mol().as_bytes()
//     }
    
// }

// impl SudtAmount {
//     pub fn to_json(&self) -> JsonUint128 {
//         JsonUint128::from(self.0)
//     }

//     // Arg needs to impl Into<Self::PackedType>
//     pub fn from_json(json_u128: JsonUint128) -> Self {
//         SudtAmount(json_u128.into())
//     }

//     pub fn to_json_bytes(&self) -> ckb_jsonrpc_types::JsonBytes {
//         self.to_mol().as_bytes().pack().into()
//     }

//     pub fn from_json_bytes(bytes: JsonBytes) -> Self {
//         Self::from_bytes(bytes.into_bytes())
//     }
// }

pub type SudtContract = Contract<OwnerLockHash, SudtAmount>;