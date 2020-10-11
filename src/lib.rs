#[macro_use]
extern crate lazy_static;

use regex::Regex;
use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::Lines;
use walkdir::WalkDir;

pub enum CCompiler {
    GCC,
    ICC,
    Clang,
}

fn ccompiler(cc: &CCompiler) -> String {
    match cc {
        CCompiler::GCC => "gcc",
        CCompiler::ICC => "icc",
        CCompiler::Clang => "clang",
    }
    .to_owned()
}

fn includes(is: Vec<&OsStr>) -> Vec<&OsStr> {
    is.iter().flat_map(|x| vec![OsStr::new("-I"), x]).collect()
}

fn cflags(cc: &CCompiler) -> Vec<&OsStr> {
    match cc {
        CCompiler::GCC => vec!["-E", "-x", "c"]
            .into_iter()
            .map(|x| OsStr::new(x))
            .collect(),
        CCompiler::ICC => vec!["-E"].into_iter().map(|x| OsStr::new(x)).collect(),
        CCompiler::Clang => vec!["-E", "-x", "c"]
            .into_iter()
            .map(|x| OsStr::new(x))
            .collect(),
    }
}

/// Preprocess using [cpphs](https://archives.haskell.org/projects.haskell.org/cpphs/).
pub fn pp_cpphs(fp: &Path, out: &Path, is: Vec<&OsStr>) {
    let os_p = fp.as_os_str();
    let out_p = out.as_os_str();
    let mut arg_vec = vec![os_p, out_p];
    for i in includes(is) {
        arg_vec.push(i);
    }
    let _ = Command::new("cpphs")
        .args(&[os_p, out_p])
        .output()
        .expect("call to C preprocessor failed");
}

fn from_file(line: &str) -> bool {
    lazy_static! {
        static ref RE: Regex = Regex::new("^# \\d+ \".*\"").unwrap();
    }
    RE.is_match(line)
}

// stateful-ish, only take lines after "src/lib.cpprs" as appropriate...
// only works w/ icc, gcc, clang
//
// could be extended to cpphs
fn begin_rust(fp: &Path, line: &str) -> bool {
    let regex_str = format!("^# \\d+ \"{}\"", fp.display());
    let re = Regex::new(&regex_str).unwrap();
    re.is_match(line)
}

fn process_lines(fp: &Path, lines: Lines) -> String {
    let mut res_vec: Vec<&str> = Vec::new();
    let mut post_src_file = false;
    for l in lines {
        if from_file(l) {
            post_src_file = false
        }
        let line_line = begin_rust(fp, l);
        if line_line {
            post_src_file = true
        }
        if post_src_file && !line_line {
            res_vec.push(l);
            res_vec.push("\n");
        }
    }
    res_vec.into_iter().collect()
}

/// Preprocess using one of the known [CCompiler](CCompiler)s
pub fn pp_cc(cc: &CCompiler, fp: &Path, out: &Path, is: &Vec<&OsStr>) {
    let os_p = fp.as_os_str();
    let mut args0 = cflags(cc);
    args0.push(os_p);
    for i in includes(is.to_vec()) {
        args0.push(i);
    }
    // FIXME: borrow?
    let cpp_res = Command::new(ccompiler(cc))
        .args(args0)
        .stdout(Stdio::piped())
        .output()
        .expect("call to C preprocessor failed");
    let raw = String::from_utf8(cpp_res.stdout).unwrap();
    let res: String = process_lines(fp, raw.lines());
    let mut out_file = File::create(out).unwrap();
    out_file.write_all(res.as_bytes()).unwrap();
}

// also I should allow users to pass -I flags lol (and -D) like cc crate?

// maybe get a .rs file name from a .cpprs file
fn as_rs(fp: &Path) -> Option<PathBuf> {
    let maybe_ext = fp.extension();
    maybe_ext.and_then({
        |p| match p.to_str().unwrap() {
            "cpprs" => {
                let main_fp = fp.file_stem().unwrap();
                let mut return_pbuf = PathBuf::from(fp.parent().unwrap());
                return_pbuf.push(main_fp);
                return_pbuf.set_extension("rs");
                Some(return_pbuf)
            }
            _ => None,
        }
    })
}

/// Preprocess all `.cpprs` files in the `src` directory
pub fn walk_src_preprocess(cc: CCompiler, include_dirs: Vec<&OsStr>) {
    walk_preprocess(cc, "src", include_dirs);
}

/// Preprocess all `.cpprs` files in a given directory.
pub fn walk_preprocess<P: AsRef<Path>>(cc: CCompiler, dir: P, include_dirs: Vec<&OsStr>) {
    let walker = WalkDir::new(dir);
    walk_preprocess_general(cc, walker, include_dirs)
}

/// Preprocess all `.cpprs` files encountered by a [WalkDir](WalkDir)
pub fn walk_preprocess_general(cc: CCompiler, walker: WalkDir, include_dirs: Vec<&OsStr>) {
    for entry in walker.into_iter().filter_map(|e| e.ok()) {
        let mdo = as_rs(entry.path());
        match mdo {
            None => (),
            Some(out_fp) => {
                println!("cargo:rerun-if-changed={}", entry.path().display());
                pp_cc(&cc, entry.path(), &out_fp, &include_dirs);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn walk_test() {
        let res = fs::remove_file("lzo-macros/src/lib.rs");
        if res.is_ok() {}
        walk_preprocess(
            CCompiler::GCC,
            "lzo-macros/src",
            vec![OsStr::new("lzo-macros/cbits")],
        )
    }
}
