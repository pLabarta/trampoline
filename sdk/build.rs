use std::path::Path;
use std::{env, fs};
fn main() {
    let bins = [
        "simple_udt",
    ];
    let out_dir = env::var_os("OUT_DIR").unwrap();
    bins.into_iter().for_each(|bin| {
        let dest_path = Path::new(&out_dir).join(bin);
        let dest_bytes = fs::read(format!("./binaries/{}", bin)).unwrap();
        assert!(!dest_bytes.is_empty());
        fs::write(&dest_path, dest_bytes)
            .expect(format!("Unable to write {} to output during build", bin).as_str());
    });
    println!("cargo:rerun-if-changed=contract/builtins/simple_udt");
}
