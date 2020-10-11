#[macro_use]
extern crate criterion;

use cppbuild::*;
use criterion::Criterion;
use std::ffi::OsStr;
use std::path::Path;

fn pp_cc_benchmark(c: &mut Criterion) {
    c.bench_function("bytecount (iterator)", |b| {
        b.iter(|| {
            pp_cc(
                &CCompiler::GCC,
                Path::new("lzo-macros/src/lib.cpprs"),
                Path::new("lzo-macros/src/lib.rs"),
                &vec![OsStr::new("lzo-macros/cbits")],
            )
        })
    });
}

criterion_group!(benches, pp_cc_benchmark);
criterion_main!(benches);
