# Loadbench

A load testing tool for benchmarking things.

Loadbench is an open-loop generator based on lightweight threads.

It aims to be minimal and flexible, providing users the ability to build performant load test benchmarks without having to rebuild the internals every time.

## Getting started

There are some examples in the repo which can be run with `cargo --example`.

To use the library, see [the documentation](https://jeffa5.github.io/loadbench).

You'll need to use a version of `rust-csv` that supports `#[serde(flatten)]`, such as putting the following in your `Cargo.toml`:

```toml
csv = { git = "https://github.com/gootorov/rust-csv", branch = "serde-flatten" }
```
