/*!

# Example

## Build

You probably want the following in your `build.rs`:

```
use cpprs::{walk_src_preprocess};

fn main() {
    walk_src_preprocess(vec![])
}
```

This will pre-process any `.cpprs` source files in `src/` using
[GCC](https://gcc.gnu.org/).

## Code

Suppose you have the following in `src/lib.cpprs`:

```c
#include <minilzo.h>

pub enum LzoError {
    LzoOk = LZO_E_OK,
    LzoError = LZO_E_ERROR,
    LzoOutOfMemory = LZO_E_OUT_OF_MEMORY,
    LzoNotCompressible = LZO_E_NOT_COMPRESSIBLE,
    LzoInputOverrun = LZO_E_INPUT_OVERRUN,
    LzoOutputOverrun = LZO_E_OUTPUT_OVERRUN,
    LzoLookbehindOverrun = LZO_E_LOOKBEHIND_OVERRUN,
    LzoEofNotFound = LZO_E_EOF_NOT_FOUND,
    LzoInputNotConsumed = LZO_E_INPUT_NOT_CONSUMED,
    LzoNotYetImplemented = LZO_E_NOT_YET_IMPLEMENTED,
    LzoInvalidArgument = LZO_E_INVALID_ARGUMENT,
    LzoInvalidAlignment = LZO_E_INVALID_ALIGNMENT,
    LzoOutputNotConsumed = LZO_E_OUTPUT_NOT_CONSUMED,
    LzoInternalError = LZO_E_INTERNAL_ERROR,
}
```

Then this will be placed in `src/lib.rs`:

```
pub enum LzoError {
    LzoOk = 0,
    LzoError = (-1),
    LzoOutOfMemory = (-2),
    LzoNotCompressible = (-3),
    LzoInputOverrun = (-4),
    LzoOutputOverrun = (-5),
    LzoLookbehindOverrun = (-6),
    LzoEofNotFound = (-7),
    LzoInputNotConsumed = (-8),
    LzoNotYetImplemented = (-9),
    LzoInvalidArgument = (-10),
    LzoInvalidAlignment = (-11),
    LzoOutputNotConsumed = (-12),
    LzoInternalError = (-99),
}
```

i.e. the macros will be filled in.

*/

#[macro_use]
extern crate lazy_static;

use regex::Regex;
use std::env;
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::str::Lines;
use walkdir::WalkDir;
use which;

pub enum CCompiler {
    GCC,
    ICC,
    Clang,
}

/// Detect C compiler, default to GCC over Clang.
fn detect_compiler() -> CCompiler {
    match which::which("gcc") {
        Ok(_) => CCompiler::GCC,
        Err(_) => match which::which("clang") {
            Ok(_) => CCompiler::Clang,
            Err(_) => match which::which("icc") {
                Ok(_) => CCompiler::ICC,
                Err(_) => panic!("No C compiler detected! Expect one of GCC, ICC, Clang"),
            },
        },
    }
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
    let mut res_vec: Vec<&str> = Vec::with_capacity(100);
    let mut post_src_file = false;
    for l in lines {
        let ff = from_file(l);
        if ff {
            post_src_file = false
        }
        // if it's not a from_file line it certainly can't be a relevant line
        let line_line = if !ff { false } else { begin_rust(fp, l) };
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

/// Preprocess, defaulting to [GCC](CCompiler::GCC) over [Clang](CCompiler::Clang).
pub fn pp(fp: &Path, out: &Path, is: &[&OsStr]) {
    let pp_guess = detect_compiler();
    pp_cc(&pp_guess, fp, out, is)
}

pub fn pp_msvc(fp: &Path, out: &Path, is: &[&OsStr]) {
    let os_p = fp.as_os_str();
    let cc = CCompiler::MSVC;
    let mut args0 = cflags(cc);
    args0.push(os_p);
    for i in includes(is.to_vec()) {
        args0.push(i);
    }
    Command::new(ccompiler(cc)).args(args0);
}

/// Preprocess using one of the known [CCompiler](CCompiler)s
pub fn pp_cc(cc: &CCompiler, fp: &Path, out: &Path, is: &[&OsStr]) {
    let os_p = fp.as_os_str();
    let mut args0 = cflags(cc);
    args0.push(os_p);
    for i in includes(is.to_vec()) {
        args0.push(i);
    }
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

/// Get includes from the `C_INCLUDE_PATH` environment variable.
fn get_include_dirs() -> Vec<OsString> {
    match env::var_os("C_INCLUDE_PATH") {
        Some(paths) => env::split_paths(&paths)
            .map(|x| x.into_os_string())
            .collect::<Vec<OsString>>(),
        None => {
            vec![]
        }
    }
}

/// Preprocess all `.cpprs` files in the `src` directory
pub fn walk_src_preprocess(include_dirs: Vec<&OsStr>) {
    walk_preprocess("src", include_dirs);
}

/// Preprocess all `.cpprs` files in a given directory.
pub fn walk_preprocess<P: AsRef<Path>>(dir: P, include_dirs: Vec<&OsStr>) {
    let walker = WalkDir::new(dir);
    let cc_guess = detect_compiler();
    walk_preprocess_general(cc_guess, walker, include_dirs)
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
        walk_preprocess("lzo-macros/src", vec![OsStr::new("lzo-macros/cbits")])
    }
}
