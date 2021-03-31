# rotary

[![Documentation](https://docs.rs/rotary/badge.svg)](https://docs.rs/rotary)
[![Crates](https://img.shields.io/crates/v/rotary.svg)](https://crates.io/crates/rotary)
[![Actions Status](https://github.com/udoprog/rotary/workflows/Rust/badge.svg)](https://github.com/udoprog/rotary/actions)

A library for working with audio buffers

The buffer is constructed similarly to a `Vec<Vec<T>>`, except the interior
vector has a fixed size. And the buffer makes no attempt to clear data which
is freed when using functions such as [Dynamic::resize].

## Formats and topologies

The following are the three canonical audio formats which are supported by
this library:
* [dynamic][Dynamic] - where each channel is stored in its own
  heap-allocated buffer.
* [interleaved][Interleaved] - where each channel is interleaved, like
  `0:0, 1:0, 1:0, 1:1`.
* [sequential][Sequential] - where each channel is stored in a linear
  buffer, one after another. Like `0:0, 0:1, 1:0, 1:0`.

These all implement the [Buf] and [BufMut] traits, allowing library authors
to abstract over any one specific format. The exact channel and frame count
of a buffer is known as its *topology*.

```rust
use rotary::BufMut as _;

let mut dynamic = rotary::dynamic![[0i16; 4]; 2];
let mut interleaved = rotary::interleaved![[0i16; 4]; 2];
let mut sequential = rotary::sequential![[0i16; 4]; 2];

dynamic.channel_mut(0).copy_from_iter(0i16..);
interleaved.channel_mut(0).copy_from_iter(0i16..);
sequential.channel_mut(0).copy_from_iter(0i16..);
```

We also support [wrapping][wrap] external buffers so that they can
interoperate like other rotary buffers.

## Example: [play-mp3]

Play an mp3 file with [minimp3-rs], [cpal], and [rubato] for resampling.

This example can handle with any channel and sample rate configuration.

```bash
cargo run --release --package rotary-examples --bin play-mp3 -- path/to/file.mp3
```

## Examples

```rust
use rand::Rng as _;

let mut buffer = rotary::Dynamic::<f32>::new();

buffer.resize_channels(2);
buffer.resize(2048);

/// Fill both channels with random noise.
let mut rng = rand::thread_rng();
rng.fill(&mut buffer[0]);
rng.fill(&mut buffer[1]);
```

For convenience we also provide several macros for constructing various
forms of dynamic audio buffers. These should mostly be used for testing.

```rust
let mut buf = rotary::Dynamic::<f32>::with_topology(4, 8);

for channel in &mut buf {
    for f in channel {
        *f = 2.0;
    }
}

assert_eq! {
    buf,
    rotary::dynamic![[2.0; 8]; 4],
};

assert_eq! {
    buf,
    rotary::dynamic![[2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0]; 4],
};
```

[play-mp3]: https://github.com/udoprog/rotary/tree/main/examples/src/bin/play-mp3.rs
[minimp3-rs]: https://github.com/germangb/minimp3-rs
[cpal]: https://github.com/RustAudio/cpal
[rubato]: https://github.com/HEnquist/rubato
[Dynamic::resize]: https://docs.rs/rotary/0/rotary/dynamic/struct.Dynamic.html#method.resize
[BitSet<u128>]: https://docs.rs/rotary/0/rotary/bit_set/struct.BitSet.html
[dynamic!]: https://docs.rs/rotary/0/rotary/macros/macro.dynamic.html
[Dynamic]: https://docs.rs/rotary/0/rotary/dynamic/struct.Dynamic.html
[Interleaved]: https://docs.rs/rotary/0/rotary/interleaved/struct.Interleaved.html
[Sequential]: https://docs.rs/rotary/0/rotary/sequential/struct.Sequential.html
[wrap]: https://docs.rs/rotary/0/rotary/wrap/index.html
[Buf]: https://docs.rs/rotaryc-re/0/rotary/trait.Buf.html
[BufMut]: https://docs.rs/rotaryc-re/0/rotary/trait.BufMut.html

License: MIT/Apache-2.0
