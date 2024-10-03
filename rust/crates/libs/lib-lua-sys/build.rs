// extern crate cc;
// use std::env;
// use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=lualib-src");
    if cfg!(any(target_os = "windows", target_os = "macos")) {
        println!("cargo:rustc-link-lib=dylib=lua");
        println!(r"cargo:rustc-link-search=native=../3rd/moon/build/bin/Release");
    }
}
