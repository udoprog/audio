# ste

[![Documentation](https://docs.rs/ste/badge.svg)](https://docs.rs/ste)
[![Crates](https://img.shields.io/crates/v/ste.svg)](https://crates.io/crates/ste)
[![Actions Status](https://github.com/udoprog/rotary/workflows/Rust/badge.svg)](https://github.com/udoprog/rotary/actions)

A single-threaded executor with some tricks up its sleeve.

This was primarily written for use in [rotary] as a low-latency way of
interacting with a single background thread for audio-related purposes, but
is otherwise a general purpose library that can be used by anyone.

**Warning:** Some of the tricks used in this crate needs to be sanity
checked for safety before you can rely on this for production uses.

## Examples

```rust
let thread = ste::Thread::new()?;

let mut n = 10;
thread.submit(|| n += 10)?;
assert_eq!(20, n);

thread.join()?;
```

[rotary]: https://github.com/udoprog/rotary

License: MIT/Apache-2.0
