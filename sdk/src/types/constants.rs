//! Hashes and default configs for CKB

#[cfg(all(feature = "std", not(feature = "script")))]
pub use ckb_sdk::constants::*;

// This constant was removed because it is available from ckb_sdk::constants
// pub const ONE_CKB: u64 = 100_000_000;

/// Standard size for hashed code data
pub const CODE_HASH_SIZE_BYTES: usize = 32;
