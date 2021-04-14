# audio-device

A library for interacting with audio devices.

The sole aim of this crate is to provide idiomatic *low level* audio
interface drivers that can be used independently. If all you need is WASAPI
or ALSA, then that is all you pay for and you should have a decent
Rust-idiomatic programming experience.

This is part of the [audio ecosystem] and makes use of core traits provided
by the [audio-core] crate.

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

[audio ecosystem] : https://docs.rs/audio
[alsa-blocking]: https://github.com/udoprog/audio/blob/main/audio-device/examples/alsa.rs
[audio-core]: https://docs.rs/audio-core
[wasapi-async]: https://github.com/udoprog/audio/blob/main/audio-device/examples/wasapi-async.rs
[wasapi-blocking]: https://github.com/udoprog/audio/blob/main/audio-device/examples/wasapi.rs

License: MIT/Apache-2.0
