use ckb_types::prelude::*;
use ckb_types::bytes::Bytes as CkBytes;
use ckb_types::packed::Bytes as PackedBytes;
use ckb_jsonrpc_types::JsonBytes;
use ckb_types::{H256, core::{Capacity, CapacityError}};
use ckb_hash::blake2b_256;
use thiserror::Error;
// use molecule::bytes::Bytes as MolBytes; // This is equivalent to CkBytes when in std mode

#[derive(Debug, Error)]
pub enum BytesError {
    #[error(transparent)]
    CapacityCalcError(#[from] CapacityError),
}

pub type BytesResult<T> = Result<T, BytesError>;

#[derive(Clone, Debug, Default)]
pub struct Bytes(pub (crate) JsonBytes);


impl Bytes {
   pub fn hash_256(&self) -> H256 {
        let raw_bytes: CkBytes = self.clone().into();
        H256(blake2b_256(raw_bytes))
    }

    pub fn len(&self) -> usize {
        let packed = PackedBytes::from(self.clone());
        packed.len()
    }

    pub fn required_capacity(&self) -> BytesResult<Capacity> {
        Capacity::bytes(self.len())
            .map_err(|e| BytesError::CapacityCalcError(e))
    }
}

impl From<CkBytes> for Bytes {
    fn from(ckbytes: CkBytes) -> Self {
        Self(ckbytes.pack().into())
    }
}

impl From<PackedBytes> for Bytes {
    fn from(packed_bytes: PackedBytes) -> Self {
        Self(packed_bytes.into())
    }
}


impl From<JsonBytes> for Bytes {
    fn from(json_bytes: JsonBytes) -> Self {
        Self(json_bytes)
    }
}

impl From<Bytes> for CkBytes {
    fn from(b: Bytes) -> Self {
        b.0.into_bytes()
    }
}

impl From<Bytes> for PackedBytes {
    fn from(b: Bytes) -> Self {
        b.0.into_bytes().pack()
    }
}

impl From<Bytes> for JsonBytes {
    fn from(b: Bytes) -> Self {
        b.0
    }
}

impl From<&Bytes> for CkBytes {
    fn from(b: &Bytes) -> Self {
        b.0.clone().into_bytes()
    }
}

impl From<&Bytes> for PackedBytes {
    fn from(b: &Bytes) -> Self {
        b.0.clone().into_bytes().pack()
    }
}

impl From<&Bytes> for JsonBytes {
    fn from(b: &Bytes) -> Self {
        b.0.clone()
    }
}
