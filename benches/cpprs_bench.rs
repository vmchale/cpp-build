#[macro_use]
extern crate criterion;

use cpprs::*;
use criterion::Criterion;
use std::ffi::OsStr;
use std::path::Path;

fn pp_cc_benchmark(c: &mut Criterion) {
    c.bench_function("lzo macros", |b| {
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
