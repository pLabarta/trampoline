#![no_std]
extern crate no_std_compat as std;

#[cfg(not(feature = "script"))]
pub mod rpc;

#[cfg(not(feature = "script"))]
pub mod chain;

#[cfg(feature = "script")]
pub mod ckb_types {
    pub use ckb_standalone_types::*;
}


#[cfg(not(feature = "script"))]
pub mod contract;


#[cfg(feature = "script")]
mod contract;
#[cfg(feature = "script")]
pub use crate::contract::{schema, builtins};




#[cfg(not(feature = "script"))]
pub mod ckb_types {
    pub use ckb_types::*;
}

// From ckb_types::conversion
#[macro_export]
macro_rules! impl_entity_unpack {
    ($original:ty, $entity:ident) => {
        use crate::ckb_types::prelude::{Unpack,  Reader, Entity};
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
        use crate::ckb_types::prelude::{Unpack, Reader, Entity};
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
        use crate::ckb_types::prelude::{Pack, Reader, Entity};
        use crate::ckb_types::bytes::Bytes;
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
        use crate::ckb_types::prelude::{Pack};
        impl Pack<$entity> for $native_type {
            fn pack(&self) -> $entity {
                $entity::new_unchecked(Bytes::from(self.to_vec()))
            }
        }
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
