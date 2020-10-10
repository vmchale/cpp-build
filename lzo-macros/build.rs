use cppbuild::{walk_src_preprocess, CCompiler};
use std::ffi::OsStr;

fn main() {
    walk_src_preprocess(CCompiler::GCC, vec![OsStr::new("cbits")]);
}
