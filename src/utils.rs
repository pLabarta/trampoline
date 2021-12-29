use anyhow::{anyhow, Result};

// Adapted from HexParser in ckb-cli/utils/arg_parser
pub fn hex_string(src: &[u8]) -> String {
    let mut buffer = vec![0; src.len() * 2];
    hex_encode(src, &mut buffer)
        .map(|_| unsafe { String::from_utf8_unchecked(buffer) })
        .expect("hex_string")
}

pub fn hex_encode(src: &[u8], dst: &mut [u8]) -> Result<()> {
    let len = src.len().checked_mul(2).unwrap();
    if dst.len() < len {
        return Err(anyhow!(
            "Invalid length in dst {}, expected: {}",
            dst.len(),
            len
        ));
    }

    hex::encode_to_slice(src, dst)?;
    Ok(())
}

pub fn hex_decode(src: &[u8], dst: &mut [u8]) -> Result<()> {
    if src.is_empty() {
        return Err(anyhow!("Invalid length in dst {}", dst.len()));
    }
    let len = dst.len().checked_mul(2).unwrap();
    if src.len() < len || ((src.len() & 1) != 0) {
        return Err(anyhow!(
            "Invalid length in dst {}, expected: {}",
            dst.len(),
            len
        ));
    }
    hex::decode_to_slice(src, dst)?;

    Ok(())
}
