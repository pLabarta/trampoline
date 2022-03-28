use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::Result;
use ckb_types::H160;
use rand::Rng;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AccountError {
    #[error("malformed or out-of-range secret key")]
    InvalidSecretKey(#[from] secp256k1::Error),

    #[error("Error occurred: {0:?}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, PartialEq)]
pub struct Account {
    sk: secp256k1::SecretKey,
    lock_arg: H160,
}

impl Account {
    pub fn from_sk(sk: secp256k1::SecretKey) -> Result<Self, AccountError> {
        let pk = secp256k1::PublicKey::from_secret_key(&ckb_crypto::secp::SECP256K1, &sk);
        // calling `unwrap` is safe here, since we pass the expected length of bytes.
        let lock_arg =
            H160::from_slice(&ckb_hash::blake2b_256(&pk.serialize()[..])[0..20]).unwrap();
        Ok(Account { sk, lock_arg })
    }

    pub fn sk_hex(&self) -> String {
        format!("{:x}", self.sk)
    }

    pub fn lock_arg_hex(&self) -> String {
        format!("{:x}", self.lock_arg)
    }

    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "sk": self.sk_hex(),
            "lock_arg": self.lock_arg_hex()
        })
    }
}

pub struct AccountManager {
    pub root_dir: PathBuf,
}

impl AccountManager {
    pub fn new<P: AsRef<Path>>(root_dir: P) -> Self {
        return AccountManager {
            root_dir: root_dir.as_ref().into(),
        };
    }

    /// Create an account, save the hex-formatted secret key to a file.
    /// The file name is the hex-formatted lock_arg.
    pub fn create_account(&self) -> Result<Account> {
        let sk = gen_secret_key();
        let account = Account::from_sk(sk)?;
        self.write_account(&account)?;
        Ok(account)
    }

    /// Import an account using hex-formatted secret key, save it to file.
    /// The file name is the hex-formatted lock_arg.
    pub fn import_account(&self, sk_string: String) -> Result<Account, AccountError> {
        let sk = secp256k1::SecretKey::from_str(&sk_string)?;
        let account = Account::from_sk(sk)?;
        self.write_account(&account)?;
        Ok(account)
    }

    fn write_account(&self, account: &Account) -> Result<(), AccountError> {
        let file = self.root_dir.join(account.lock_arg_hex());
        fs::write(file, account.sk_hex())?;
        Ok(())
    }
}

fn gen_secret_key() -> secp256k1::SecretKey {
    let mut seed = vec![0; 32];
    let mut rng = rand::thread_rng();
    loop {
        rng.fill(seed.as_mut_slice());
        if let Ok(key) = secp256k1::SecretKey::from_slice(&seed) {
            return key;
        }
    }
}

pub fn lock_arg(sk: secp256k1::SecretKey) -> Result<String> {
    let pk = secp256k1::PublicKey::from_secret_key(&ckb_crypto::secp::SECP256K1, &sk);
    let lock_arg = H160::from_slice(&ckb_hash::blake2b_256(&pk.serialize()[..])[0..20])?;
    Ok(format!("{:x}", lock_arg))
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn test_new_account() -> Result<()> {
        let root = tempdir()?;
        let mgr = AccountManager::new(&root);
        let account = mgr.create_account()?;
        let sk_hex = fs::read_to_string(root.path().join(account.lock_arg_hex()))?;
        let sk = secp256k1::SecretKey::from_str(&sk_hex)?;
        let account2 = Account::from_sk(sk)?;
        assert_eq!(account2, account);
        Ok(())
    }

    #[test]
    fn test_import_account() {
        let root = tempdir().unwrap();
        let mgr = AccountManager::new(&root);

        let sk_hex = "009c0df368efef6084ba35ded33f05ef2a5f4b25d7841bce77e2449be9311dba";
        let account = mgr.import_account(sk_hex.into()).unwrap();
        let sk_file = root.path().join(account.lock_arg_hex());
        assert!(sk_file.exists());
        assert_eq!(fs::read_to_string(sk_file).unwrap(), sk_hex);
    }

    #[test]
    fn test_import_invalid_account() {
        let root = tempdir().unwrap();
        let mgr = AccountManager::new(&root);

        let sk_hex = "009c0df368efef6084ba35ded00000ef2a5f4b25d7841bce77e2449be9311dbx";
        let result = mgr.import_account(sk_hex.into());
        assert!(result.is_err());
    }

    #[test]
    fn test_to_json() {
        let sk = secp256k1::SecretKey::from_str(
            "009c0df368efef6084ba35ded33f05ef2a5f4b25d7841bce77e2449be9311dba",
        )
        .unwrap();
        let account = Account::from_sk(sk).unwrap();
        let json = account.to_json();
        assert_eq!(
            json["sk"],
            json!("009c0df368efef6084ba35ded33f05ef2a5f4b25d7841bce77e2449be9311dba")
        );
        assert_eq!(
            json["lock_arg"],
            json!("277940df3084136576140e7fa07c3961f0c4cca3")
        );
    }
}
