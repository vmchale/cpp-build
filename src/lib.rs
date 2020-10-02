use std::env;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

// should work with pgcc, icc, clang, gcc...
fn pp_cc(cc: &str, fp: &Path, out: &Path) -> () {
    let os_p = fp.as_os_str();
    let cpp_res = Command::new(cc)
        .args(&[OsStr::new("-E"), os_p])
        .stdout(Stdio::piped())
        .output()
        .expect("call to C preprocessor failed");
    let res = String::from_utf8(cpp_res.stdout).unwrap();
    let mut out_file = File::create(out).unwrap();
    out_file.write_all(res.as_bytes()).unwrap();
}

fn as_rs(fp: &Path) -> Option<PathBuf> {
    let maybe_ext = fp.extension();
    None
}
