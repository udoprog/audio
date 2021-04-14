# audio

[![Documentation](https://docs.rs/audio/badge.svg)](https://docs.rs/audio)
[![Crates](https://img.shields.io/crates/v/audio.svg)](https://crates.io/crates/audio)
[![Actions Status](https://github.com/udoprog/audio/workflows/Rust/badge.svg)](https://github.com/udoprog/audio/actions)

A crate for working with audio in Rust.

This is made up of several parts, each can be used independently of each other:

* [audio-core] - which defines core traits that allows for safely
  interacting with audio buffers.
* [audio] - this crate, which provides a collection of high-quality audio
  buffers which implements the traits provided in [audio-core].
* [audio-device] - a crate for interacting with audio devices in idiomatic
  Rust.
* [audio-generator] - crate for generating audio.

Audio buffers provided by this crate are conceptually kinda like
`Vec<Vec<T>>`, except the interior vector has a fixed size. And the buffer
makes no attempt to *clear data* which is freed when using resizing
functions such as [Dynamic::resize].

## Formats and topologies

The following are the three canonical audio formats which are supported by
this crate:
* [dynamic][Dynamic] - where each channel is stored in its own
  heap-allocated buffer.
* [interleaved][Interleaved] - where each channel is interleaved, like
  `0:0, 1:0, 1:0, 1:1`.
* [sequential][Sequential] - where each channel is stored in a linear
  buffer, one after another. Like `0:0, 0:1, 1:0, 1:0`.

These all implement the [Channels] and [ChannelsMut] traits, allowing
library authors to abstract over any one specific format. The exact channel
and frame count of a buffer is known as its *topology*.

```rust
use audio::ChannelsMut as _;

let mut dynamic = audio::dynamic![[0i16; 4]; 2];
let mut interleaved = audio::interleaved![[0i16; 4]; 2];
let mut sequential = audio::sequential![[0i16; 4]; 2];

dynamic.channel_mut(0).copy_from_iter(0i16..);
interleaved.channel_mut(0).copy_from_iter(0i16..);
sequential.channel_mut(0).copy_from_iter(0i16..);
```

We also support [wrapping][wrap] external buffers so that they can
interoperate like other audio buffers.

## Example: [play-mp3]

Play an mp3 file with [minimp3-rs], [cpal], and [rubato] for resampling.

This example can handle with any channel and sample rate configuration.

```bash
cargo run --release --package audio-examples --bin play-mp3 -- path/to/file.mp3
```

## Examples

```rust
use rand::Rng as _;

let mut buffer = audio::Dynamic::<f32>::new();

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
let mut buf = audio::Dynamic::<f32>::with_topology(4, 8);

for channel in &mut buf {
    for f in channel {
        *f = 2.0;
    }
}

assert_eq! {
    buf,
    audio::dynamic![[2.0; 8]; 4],
};

assert_eq! {
    buf,
    audio::dynamic![[2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0, 2.0]; 4],
};
```

[audio]: https://docs.rs/audio/0.2.0-alpha.4
[audio-core]: https://docs.rs/audio-core/0.2.0-alpha.4
[audio-device]: https://docs.rs/audio-device/0.2.0-alpha.4
[audio-generator]: https://docs.rs/audio-generator/0.2.0-alpha.4
[Channels]: https://docs.rs/audio-core/0.2.0-alpha.4/audio_core/trait.Channels.html
[ChannelsMut]: https://docs.rs/audio-core/0.2.0-alpha.4/audio_core/trait.ChannelsMut.html
[cpal]: https://github.com/RustAudio/cpal
[Dynamic::resize]: https://docs.rs/audio/0.2.0-alpha.4/audio/dynamic/struct.Dynamic.html#method.resize
[dynamic!]: https://docs.rs/audio/0.2.0-alpha.4/audio/macros/macro.dynamic.html
[Dynamic]: https://docs.rs/audio/0.2.0-alpha.4/audio/dynamic/struct.Dynamic.html
[Interleaved]: https://docs.rs/audio/0.2.0-alpha.4/audio/interleaved/struct.Interleaved.html
[minimp3-rs]: https://github.com/germangb/minimp3-rs
[play-mp3]: https://github.com/udoprog/audio/tree/main/examples/src/bin/play-mp3.rs
[rubato]: https://github.com/HEnquist/rubato
[Sequential]: https://docs.rs/audio/0.2.0-alpha.4/audio/sequential/struct.Sequential.html
[wrap]: https://docs.rs/audio/0.2.0-alpha.4/audio/wrap/index.html

License: MIT/Apache-2.0
