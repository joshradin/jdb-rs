use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=testFixtures/BusyBeaver.java");

    let out_dir =
        PathBuf::from(env::var("OUT_DIR").expect("$OUT_DIR not set. Please build with cargo"));

    let status = Command::new("javac")
        .arg("-d")
        .arg(&out_dir)
        .arg("testFixtures/BusyBeaver.java")
        .status()
        .expect("Failed to execute java");
    if !status.success() {
        panic!("Failed to execute java");
    }
}
