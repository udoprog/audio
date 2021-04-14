# audio-device

A library for interacting with audio devices.

This is intended to provide both blocking and non-blocking idiomatic audio
drivers for all tier 1 platforms and systems (see list below).

The sole aim of this crate is to provide idiomatic *low level* audio
interface drivers that can be used independently of the larger system. If
all you need is WASAPI or ALSA, then that is all you pay for and you should
have a decent Rust-idiomatic programming experience.

This also makes use of the core traits provided by the [audio-core] crate.

## Examples

* [ALSA blocking playback][alsa-blocking].
* [WASAPI blocking playback][wasapi-blocking].
* [WASAPI async playback][wasapi-async].

## Support

Supported tier 1 platforms and systems are the following:

| Platform | System | Blocking | Async   |
|----------|--------|----------|---------|
| Windows  | WASAPI | **wip**  | **wip** |
| Linux    | ALSA   | **wip**  | **wip** |

[audio-core]: https://docs.rs/audio-core
[alsa-blocking]: https://github.com/udoprog/audio/blob/main/audio-device/examples/alsa.rs
[wasapi-blocking]: https://github.com/udoprog/audio/blob/main/audio-device/examples/wasapi.rs
[wasapi-async]: https://github.com/udoprog/audio/blob/main/audio-device/examples/wasapi-async.rs

License: MIT/Apache-2.0
