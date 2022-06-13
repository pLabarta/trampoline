pub mod account;
pub mod transaction;
pub mod lock;
pub mod rpc;
pub mod hex;
mod error;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
