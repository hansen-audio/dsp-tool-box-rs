# DSP Tool Box for Rust

[![Rust](https://github.com/hansen-audio/dsp-tool-box-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/hansen-audio/dsp-tool-box-rs/actions/workflows/rust.yml)

## Summary

The ```dsp-tool-box-rs``` contains very basic DSP algorithms. Algorithms can operate on one sample at a time. Like this an update of a parameter can be done any time while processing.

## Building the project

Install [Rust](https://rustup.rs/)!

Execute the following commands on cli.

```
git clone https://www.github.com/hansen-audio/dsp-tool-box-rs.git
cd dsp-tool-box-rs
cargo build
cargo test
```

## Algorithms

Currently the following algorithms are available:

* modulation phase
* one pole filter

## Using the algorithms

```Rust
let mut filter = OnePoleFilter::new(0.9);
let out = filter.process(1.);
println("{:?}", out);
```

## License

Copyright 2021 Hansen Audio

Licensed under the MIT: https://mit-license.org/
