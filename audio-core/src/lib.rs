//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/audio-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/audio)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/audio-core.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/audio-core)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-audio--core-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/audio-core)
//! [<img alt="build status" src="https://img.shields.io/github/workflow/status/udoprog/audio/CI/main?style=for-the-badge" height="20">](https://github.com/udoprog/audio/actions?query=branch%3Amain)
//!
//! The core [audio] traits.
//!
//! If you want to build an audio component that is completely agnostic to the
//! shape of any one given audio buffer you can add a dependency directly to
//! these traits instead of depending on all of the [audio] crate.
//!
//! [audio]: https://docs.rs/audio

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
#![allow(clippy::should_implement_trait)]

pub mod buf;
pub use self::buf::*;

mod buf_mut;
pub use self::buf_mut::BufMut;

mod channel;
pub use self::channel::Channel;

mod channel_mut;
pub use self::channel_mut::ChannelMut;

mod frame;
pub use self::frame::Frame;

mod frame_mut;
pub use self::frame_mut::FrameMut;

pub mod translate;
pub use self::translate::Translate;

mod sample;
pub use self::sample::Sample;

mod read_buf;
pub use self::read_buf::ReadBuf;

mod write_buf;
pub use self::write_buf::WriteBuf;

mod exact_size_buf;
pub use self::exact_size_buf::ExactSizeBuf;

mod resizable_buf;
pub use self::resizable_buf::ResizableBuf;

mod interleaved_buf;
pub use self::interleaved_buf::InterleavedBuf;

mod interleaved_buf_mut;
pub use self::interleaved_buf_mut::InterleavedBufMut;

mod linear_channel;
pub use self::linear_channel::LinearChannel;

mod linear_channel_mut;
pub use self::linear_channel_mut::LinearChannelMut;

mod uniform_buf;
pub use self::uniform_buf::UniformBuf;
