use crate::compose::Service;
use std::io::{self, Write};
use std::process::Command;

pub fn init_ckb_volume(volume_name: &str) {
    // Create a named volume
    let mut create_volume = Command::new("docker");
    create_volume.arg("volume").arg("create").arg(&volume_name);
    let volume_output = create_volume.output().expect("failed to create volume");
    io::stdout().write_all(&volume_output.stdout).unwrap();
    // Init a CKB dev chain in that volume
    let mut init_volume = Command::new("docker");
    init_volume
        .arg("run")
        .arg("-v")
        .arg(format!("{}:/var/ckb/lib", &volume_name))
        .arg("-e")
        .arg("\"CKB_CHAIN=dev\"")
        .arg("nervos/ckb:latest")
        .arg("init");
    let init_output = init_volume.output().expect("failed to init CKB in volume");
    io::stdout().write_all(&init_output.stdout).unwrap();
}

impl Service {
    pub fn node() {
        Service
    }
}
