#[cfg(not(feature = "script"))]
pub use ckb_jsonrpc_types::JsonBytes;

use std::marker::PhantomData;
use std::prelude::v1::*;
#[cfg(feature = "script")]
pub struct JsonBytes(crate::ckb_types::bytes::Bytes);
#[cfg(feature = "script")]
impl JsonBytes {
    pub fn from_bytes(bytes: Bytes) -> Self {
        Self(bytes)
    }
    pub fn into_bytes(self) -> Bytes {
        self.0
    }
}

#[cfg(feature = "script")]
impl From<JsonBytes> for ckb_standalone_types::packed::Bytes {
    fn from(bytes: JsonBytes) -> Self {
        bytes.into_bytes().pack()
    }
}

#[cfg(feature = "script")]
impl From<ckb_standalone_types::packed::Bytes> for JsonBytes {
    fn from(bytes: ckb_standalone_types::packed::Bytes) -> Self {
        Self(bytes.unpack())
    }
}

use crate::ckb_types::{bytes::Bytes, prelude::*};

pub trait JsonByteConversion {
    fn to_json_bytes(&self) -> JsonBytes;
    fn from_json_bytes(bytes: JsonBytes) -> Self;
}

pub trait JsonConversion {
    type JsonType;
    fn to_json(&self) -> Self::JsonType;

    fn from_json(json: Self::JsonType) -> Self;
}

pub trait MolConversion {
    type MolType: Entity;

    fn to_mol(&self) -> Self::MolType;

    fn from_mol(entity: Self::MolType) -> Self;
}

pub trait BytesConversion: MolConversion {
    fn from_bytes(bytes: Bytes) -> Self;

    fn to_bytes(&self) -> Bytes;
}

// TO DO: Think about the tradeoffs of deriving these traits?
// This is a wrapper type for schema primitive types that works
// for all primitives that have conversion trait implemented.
// Saves from having to implement mol conversion traits etc
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct SchemaPrimitiveType<T, M> {
    pub inner: T,
    _entity_type: std::marker::PhantomData<M>,
}

impl<T, M> SchemaPrimitiveType<T, M>
where
    M: Entity + Unpack<T>,
    T: Pack<M>,
{
    pub fn byte_size(&self) -> usize {
        self.to_mol().as_builder().expected_length()
    }
}
impl<T, M> SchemaPrimitiveType<T, M> {
    pub fn new(inner: T) -> Self {
        Self {
            inner,
            _entity_type: std::marker::PhantomData::<M>,
        }
    }

    pub fn from(native_type: T) -> Self {
        SchemaPrimitiveType::new(native_type)
    }

    pub fn into(self) -> T {
        self.inner
    }
}

impl<T, M> MolConversion for SchemaPrimitiveType<T, M>
where
    M: Entity + Unpack<T>,
    T: Pack<M>,
{
    type MolType = M;
    fn to_mol(&self) -> Self::MolType {
        self.inner.pack()
    }

    fn from_mol(entity: Self::MolType) -> Self {
        Self {
            inner: entity.unpack(),
            _entity_type: std::marker::PhantomData::<M>,
        }
    }
}

impl<T, M> BytesConversion for SchemaPrimitiveType<T, M>
where
    M: Entity + Unpack<T>,
    T: Pack<M>,
{
    fn from_bytes(bytes: Bytes) -> Self {
        Self {
            inner: M::from_compatible_slice(bytes.as_ref())
                .expect("Unable to build primitive type from bytes")
                .unpack(),
            _entity_type: PhantomData::<M>,
        }
    }

    fn to_bytes(&self) -> Bytes {
        self.to_mol().as_bytes()
    }
}

impl<T, M> JsonByteConversion for SchemaPrimitiveType<T, M>
where
    M: Entity + Unpack<T>,
    T: Pack<M>,
{
    fn to_json_bytes(&self) -> JsonBytes {
        self.to_mol().as_bytes().pack().into()
    }

    fn from_json_bytes(bytes: JsonBytes) -> Self {
        Self::from_bytes(bytes.into_bytes())
    }
}

impl<T, M> JsonConversion for SchemaPrimitiveType<T, M>
where
    M: Entity + Unpack<T>,
    T: Pack<M>,
{
    type JsonType = JsonBytes;

    fn to_json(&self) -> Self::JsonType {
        self.to_json_bytes()
    }

    fn from_json(json: Self::JsonType) -> Self {
        Self::from_json_bytes(json)
    }
}
