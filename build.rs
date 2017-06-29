extern crate bindgen;
extern crate gcc;
extern crate regex;

use std::env;
use std::fs;
use std::path::PathBuf;
use std::io::{ Write, BufWriter };
use std::process::Command;

use regex::Regex;

static OR_ROOT_DIR: &str = "./openreil/";

fn main() {
    // Generate bindings first
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .enable_cxx_namespaces()
        .ctypes_prefix("libc")
        .generate_comments(true)
        .unstable_rust(true)
        .generate()
        .expect("Could not generate bindings")
        .to_string();
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_path = PathBuf::from(out_dir.as_str());

    let mut modified_bindings = String::with_capacity(bindings.len());

    let rgx = Regex::new(r"pub const (?:__?[A-Z]+_?[A-Z]+|.*_MIN\b|.*_MAX\b)").expect("could not compile regex");

    for line in bindings.lines() {
        if line.starts_with("pub mod root") {
            modified_bindings.push_str(line);
            modified_bindings.push_str("\n    use self::super::libc;\n");
        } else if ! rgx.is_match(line) {
            modified_bindings.push_str(line);
            modified_bindings.push('\n');
        }
    }

    let bindings = modified_bindings;

    let outfile = fs::File::create(out_path.join("bindings.rs")).expect("Could not create file");
    let mut out_buf = BufWriter::new(outfile);

    out_buf.write_all(bindings.as_bytes()).expect("Could not write to outfile");

    // Build libopenreil
    let autogen = Command::new("sh")
        .current_dir(OR_ROOT_DIR)
        .arg("-c")
        .arg("./autogen.sh")
        .status()
        .expect("failed to execute autogen.sh");
    if !autogen.success() { panic!("autogen.sh did not exit successfully") }

    let configure = Command::new("sh")
        .current_dir(OR_ROOT_DIR)
        .arg("-c")
        .arg("./configure")
        .status()
        .expect("failed to execute configure");
    if !configure.success() { panic!("configure did not exit successfully") }

    let make = Command::new("sh")
        .current_dir(OR_ROOT_DIR)
        .arg("-c")
        .arg("make")
        .status()
        .expect("failed to execute make");
    if !make.success() { panic!("make did not exit successfully") }

    let copy = Command::new("cp")
        .arg("./openreil/libopenreil/src/libopenreil.a")
        .arg(&out_dir)
        .status()
        .expect("failed to execute cp");
    if !copy.success() { panic!("could not copy library") }

    println!("cargo:rustc-flags=-l dylib=stdc++");
    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=openreil");
}
