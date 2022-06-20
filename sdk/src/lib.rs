#![no_std]
extern crate no_std_compat as std;

#[cfg(not(feature = "script"))]
pub mod account;
#[cfg(not(feature = "script"))]
pub mod chain;
#[cfg(not(feature = "script"))]
pub mod rpc;

pub mod contract;

pub (crate) mod types;

pub use types::{bytes, cell, constants, script};

#[cfg(not(feature = "script"))]
pub use types::{query, transaction, address};

#[cfg(feature = "script")]
#[macro_export]
macro_rules! impl_std_convert {
    ($name:ident, $bytes_size:expr) => {
        impl ::core::convert::AsRef<[u8]> for $name {
            #[inline]
            fn as_ref(&self) -> &[u8] {
                &self.0[..]
            }
        }
        impl ::core::convert::AsMut<[u8]> for $name {
            #[inline]
            fn as_mut(&mut self) -> &mut [u8] {
                &mut self.0[..]
            }
        }
        impl ::core::convert::From<[u8; $bytes_size]> for $name {
            #[inline]
            fn from(bytes: [u8; $bytes_size]) -> Self {
                $name(bytes)
            }
        }
        impl ::core::convert::From<$name> for [u8; $bytes_size] {
            #[inline]
            fn from(hash: $name) -> Self {
                hash.0
            }
        }
    };
}

pub mod ckb_types {
    #[cfg(feature = "script")]
    pub use ckb_standalone_types::{self, prelude, bytes, packed};
    #[cfg(feature = "script")]
    pub use ckb_standalone_types::prelude::{Pack, Unpack, PackVec, Entity, Reader, Builder};
 

    #[cfg(feature = "script")]
    pub mod core {
        pub use ckb_standalone_types::core::*;
        pub use ckb_standalone_types::packed::*;
        use super::prelude::*;
        use crate::impl_std_convert;
        mod capacity {
            use std::prelude::v1::*;
            

            
            /// CKB capacity.
            ///
            /// It is encoded as the amount of `Shannons` internally.
            #[derive(
                Debug, Clone, Copy, Default, Hash, PartialEq, Eq, PartialOrd, Ord
            )]
            pub struct Capacity(u64);

            /// Represents the ratio `numerator / denominator`, where `numerator` and `denominator` are both
            /// unsigned 64-bit integers.
            #[derive(Clone, PartialEq, Debug, Eq, Copy)]
            pub struct Ratio {
                /// Numerator.
                numer: u64,
                /// Denominator.
                denom: u64,
            }

            impl Ratio {
                /// Creates a ratio numer / denom.
                pub const fn new(numer: u64, denom: u64) -> Self {
                    Self { numer, denom }
                }

                /// The numerator in ratio numerator / denominator.
                pub fn numer(&self) -> u64 {
                    self.numer
                }

                /// The denominator in ratio numerator / denominator.
                pub fn denom(&self) -> u64 {
                    self.denom
                }
            }

            /// Conversion into `Capacity`.
            pub trait IntoCapacity {
                /// Converts `self` into `Capacity`.
                fn into_capacity(self) -> Capacity;
            }

            impl IntoCapacity for Capacity {
                fn into_capacity(self) -> Capacity {
                    self
                }
            }

            impl IntoCapacity for u64 {
                fn into_capacity(self) -> Capacity {
                    Capacity::shannons(self)
                }
            }

            impl IntoCapacity for u32 {
                fn into_capacity(self) -> Capacity {
                    Capacity::shannons(u64::from(self))
                }
            }

            impl IntoCapacity for u16 {
                fn into_capacity(self) -> Capacity {
                    Capacity::shannons(u64::from(self))
                }
            }

            impl IntoCapacity for u8 {
                fn into_capacity(self) -> Capacity {
                    Capacity::shannons(u64::from(self))
                }
            }

            // A `Byte` contains how many `Shannons`.
            const BYTE_SHANNONS: u64 = 100_000_000;

        
            // impl ::std::fmt::Display for Error {
            //     fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            //         write!(f, "OccupiedCapacity: overflow")
            //     }
            // }

            pub type CapacityError = String;
            /// Numeric operation result.
            pub type CapacityResult<T> = core::result::Result<T, String>;

            impl Capacity {
                /// Capacity of zero Shannons.
                pub const fn zero() -> Self {
                    Capacity(0)
                }

                /// Capacity of one Shannon.
                pub const fn one() -> Self {
                    Capacity(1)
                }

                /// Views the capacity as Shannons.
                pub const fn shannons(val: u64) -> Self {
                    Capacity(val)
                }

                /// Views the capacity as CKBytes.
                pub fn bytes(val: usize) -> CapacityResult<Self> {
                    (val as u64)
                        .checked_mul(BYTE_SHANNONS)
                        .map(Capacity::shannons)
                        .ok_or("Overflow".to_string())
                }

                /// Views the capacity as Shannons.
                pub fn as_u64(self) -> u64 {
                    self.0
                }

                /// Adds self and rhs and checks overflow error.
                pub fn safe_add<C: IntoCapacity>(self, rhs: C) -> CapacityResult<Self> {
                    self.0
                        .checked_add(rhs.into_capacity().0)
                        .map(Capacity::shannons)
                        .ok_or("Overflow".to_string())
                }

                /// Subtracts self and rhs and checks overflow error.
                pub fn safe_sub<C: IntoCapacity>(self, rhs: C) -> CapacityResult<Self> {
                    self.0
                        .checked_sub(rhs.into_capacity().0)
                        .map(Capacity::shannons)
                        .ok_or("Overflow".to_string())
                }

                /// Multiplies self and rhs and checks overflow error.
                pub fn safe_mul<C: IntoCapacity>(self, rhs: C) -> CapacityResult<Self> {
                    self.0
                        .checked_mul(rhs.into_capacity().0)
                        .map(Capacity::shannons)
                        .ok_or("Overflow".to_string())
                }

                /// Multiplies self with a ratio and checks overflow error.
                pub fn safe_mul_ratio(self, ratio: Ratio) -> CapacityResult<Self> {
                    self.0
                        .checked_mul(ratio.numer())
                        .and_then(|ret| ret.checked_div(ratio.denom()))
                        .map(Capacity::shannons)
                        .ok_or("Overflow".to_string())
                }
            }

            impl std::str::FromStr for Capacity {
                type Err = CapacityError;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    Ok(Capacity(s.parse::<u64>().map_err(|e| "ParseInt Error".to_string())?))
                }
            }

            // impl ::std::fmt::Display for Capacity {
            //     fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            //         self.0.fmt(f)
            //     }
            // }

            // impl ::std::fmt::LowerHex for Capacity {
            //     fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
            //         self.0.fmt(f)
            //     }
            // }

        }
        pub use capacity::*;
    }
    
    
    #[cfg(feature = "script")]
    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub struct H160(pub [u8; 20]);

    /// The 32-byte fixed-length binary data.
    ///
    /// The name comes from the number of bits in the data.
    ///
    /// In JSONRPC, it is encoded as a 0x-prefixed hex string.
    #[cfg(feature = "script")]
    #[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
    pub struct H256(pub [u8; 32]);

    #[cfg(feature = "script")]
    impl_std_convert!(H160, 20);

    #[cfg(feature = "script")]
    impl_std_convert!(H256, 32);




    #[cfg(not(feature = "script"))]
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


