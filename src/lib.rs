use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use walkdir::WalkDir;

/// should work with pgcc, icc, clang, gcc...
pub fn pp_cc(cc: &str, fp: &Path, out: &Path) -> () {
    let os_p = fp.as_os_str();
    let cpp_res = Command::new(cc)
        .args(&[OsStr::new("-E"), OsStr::new("-x"), OsStr::new("c"), os_p])
        .stdout(Stdio::piped())
        .output()
        .expect("call to C preprocessor failed");
    let res = String::from_utf8(cpp_res.stdout).unwrap();
    let mut out_file = File::create(out).unwrap();
    out_file.write_all(res.as_bytes()).unwrap();
}
// also I should allow users to pass -I flags lol (and -D) like cc crate?

fn as_rs(fp: &Path) -> Option<PathBuf> {
    let maybe_ext = fp.extension();
    maybe_ext.and_then({
        |p| match p.to_str().unwrap() {
            "cpprs" => {
                let main_fp = fp.file_stem().unwrap();
                let mut return_pbuf = PathBuf::from(fp.parent().unwrap());
                return_pbuf.push(main_fp);
                return_pbuf.set_extension("rs");
                return Some(return_pbuf);
            }
            _ => return None,
        }
    })
}

pub fn walk_src(cc: &str) -> () {
    walk_dir(cc, "src");
}

fn walk_dir<P: AsRef<Path>>(cc: &str, dir: P) -> () {
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let mdo = as_rs(entry.path());
        match mdo {
            None => (),
            Some(out_fp) => pp_cc(cc, entry.path(), &out_fp),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn walk_test() {
        walk_dir("gcc", "test/demo")
    }
}
