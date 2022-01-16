use includedir_codegen::Compression;
use std::path::Path;
use std::{env, fs};
fn main() {
    includedir_codegen::start("DAPP_FILES")
        .dir("templates/", Compression::Gzip)
        .build("templates.rs")
        .unwrap();
}
