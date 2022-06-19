pub mod rpc;

pub mod chain;

pub mod contract;

pub mod account;

pub mod types;

pub mod ckb_types {
    #[cfg(no_std)]
    pub use ckb_standalone_types::*;
    #[cfg(not(no_std))]
    pub use ckb_types::*;
}

pub mod precompiled {
    pub const SUDT: &[u8] = include_bytes!("../binaries/simple_udt");
    pub const TNFT: &[u8] = include_bytes!("../binaries/trampoline-nft");
}

// From ckb_types::conversion
#[macro_export]
macro_rules! impl_entity_unpack {
    ($original:ty, $entity:ident) => {
        impl Unpack<$original> for $entity {
            fn unpack(&self) -> $original {
                self.as_reader().unpack()
            }
        }
    };
}
#[macro_export]
macro_rules! impl_primitive_reader_unpack {
    ($original:ty, $entity:ident, $size:literal, $byte_method:ident) => {
        impl Unpack<$original> for $entity<'_> {
            fn unpack(&self) -> $original {
                let mut b = [0u8; $size];
                b.copy_from_slice(self.as_slice());
                <$original>::$byte_method(b)
            }
        }
    };
    ($original:ty, $entity:ident, $size:literal) => {
        impl Unpack<$original> for $entity<'_> {
            fn unpack(&self) -> $original {
                let mut b = [0u8; $size];
                b.copy_from_slice(self.as_slice());
                <$original>::from_le_bytes(b)
            }
        }
    };
}
#[macro_export]
macro_rules! impl_pack_for_primitive {
    ($native_type:ty, $entity:ident) => {
        impl Pack<$entity> for $native_type {
            fn pack(&self) -> $entity {
                $entity::new_unchecked(Bytes::from(self.to_le_bytes().to_vec()))
            }
        }
    };
}

#[macro_export]
macro_rules! impl_pack_for_fixed_byte_array {
    ($native_type:ty, $entity:ident) => {
        impl Pack<$entity> for $native_type {
            fn pack(&self) -> $entity {
                $entity::new_unchecked(Bytes::from(self.to_vec()))
            }
        }
    };
}