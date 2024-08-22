// extern crate cc;
// use std::env;
// use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=lualib-src");
    if cfg!(target_os = "windows"){
        println!("cargo:rustc-link-lib=dylib=lua");
        println!(r"cargo:rustc-link-search=native=../3rd/moon/build/bin/Release");
    }
}
