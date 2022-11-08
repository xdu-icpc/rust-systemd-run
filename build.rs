use std::env;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let st = Command::new("make")
        .args(["-C", "test-aux"])
        .arg(&format!("OUT_DIR={}", out_dir))
        .status()
        .unwrap();
    if !st.success() {
        panic!("make -C test-aux failed");
    }
}
