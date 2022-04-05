use ckb_types::core::Capacity;
use ckb_types::bytes::Bytes as CkBytes;
use ckb_types::packed::Bytes as PackedBytes;
use ckb_jsonrpc_types::JsonBytes;

pub mod transaction;
pub mod cell;
pub mod script;
pub mod bytes;
pub mod constants;

// TO DO: Implement this trait for all types

pub trait TrampolineBaseType: Into<CkBytes> + Into<PackedBytes> + Into<JsonBytes> {
    type Error: std::error::Error;
    fn validate(&self) -> Result<(), Self::Error>;

    fn required_capacity(&self) -> Result<Capacity, Self::Error>;

    fn size_bytes(&self) -> Result<usize, Self::Error>;
}