use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use walkdir::WalkDir;

pub enum CCompiler {
    GCC,
    ICC,
    Clang,
    PGCC,
}

fn ccompiler(cc: &CCompiler) -> String {
    match cc {
        CCompiler::GCC => "gcc",
        CCompiler::ICC => "icc",
        CCompiler::Clang => "clang",
        CCompiler::PGCC => "pgcc",
    }
    .to_owned()
}

fn cflags(cc: &CCompiler) -> Vec<&OsStr> {
    match cc {
        CCompiler::GCC => vec!["-E", "-x", "c"]
            .into_iter()
            .map(|x| OsStr::new(x))
            .collect(),
        CCompiler::ICC => vec!["-E"].into_iter().map(|x| OsStr::new(x)).collect(),
        CCompiler::PGCC => vec!["-E"].into_iter().map(|x| OsStr::new(x)).collect(),
        CCompiler::Clang => vec!["-E", "-x", "c"]
            .into_iter()
            .map(|x| OsStr::new(x))
            .collect(),
    }
}

/// Preprocess using [cpphs](https://archives.haskell.org/projects.haskell.org/cpphs/).
pub fn pp_cpphs(fp: &Path, out: &Path) {
    let os_p = fp.as_os_str();
    let out_p = out.as_os_str();
    let _ = Command::new("cpphs")
        .args(&[os_p, out_p])
        .output()
        .expect("call to C preprocessor failed");
}

pub fn pp_cc(cc: &CCompiler, fp: &Path, out: &Path) {
    let os_p = fp.as_os_str();
    let mut args0 = cflags(cc);
    args0.push(os_p);
    // FIXME: borrow?
    let cpp_res = Command::new(ccompiler(cc))
        .args(args0) // FIXME: don't pass -x c for pgcc (clang, gcc need it)
        .stdout(Stdio::piped())
        .output()
        .expect("call to C preprocessor failed");
    let res = String::from_utf8(cpp_res.stdout).unwrap();
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

pub fn walk_src_preprocess(cc: CCompiler) {
    walk_preprocess(cc, "src");
}

/// This function walks a given directory
pub fn walk_preprocess<P: AsRef<Path>>(cc: CCompiler, dir: P) {
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let mdo = as_rs(entry.path());
        match mdo {
            None => (),
            Some(out_fp) => pp_cc(&cc, entry.path(), &out_fp),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn walk_test() {
        let res = fs::remove_file("test/demo/test.rs");
        match res {
            Ok(_) => (),
            Err(_) => (),
        };
        walk_preprocess(CCompiler::GCC, "test/demo")
    }
}
