use std::path::Path;
use std::{env, fs};
fn main() {

    let bins = ["simple_udt", "m_nft/class-type", "m_nft/issuer-type", "m_nft/nft-type"];
    let out_dir = env::var_os("OUT_DIR").unwrap();
    fs::create_dir(Path::new(&out_dir).join("m_nft"));
    bins.into_iter().for_each(|bin| {
        
        let dest_path = Path::new(&out_dir).join(bin);
        let dest_bytes = fs::read(format!("./binaries/{}", bin)).unwrap();
        assert!(!dest_bytes.is_empty());
        fs::write(&dest_path, dest_bytes).expect(format!("Unable to write {} to output during build", bin).as_str());
    });
}
