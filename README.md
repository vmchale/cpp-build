# cpp-rs

Put this in your `build.rs` to use the C pre-processor with Rust.

## Example Use

```rust
fn main() {
    walk_dir("src")
}
```

This will pre-process any `.cpprs` source files in `src/` using
the system C compiler.

See the [lzo-macros](https://github.com/vmchale/cpp-build/tree/main/lzo-macros)
example.

## Known Defects

The C pre-processor will discard any lines beginning with `#`, so that e.g.

```
#[macro_use]
```

would be thrown away.
