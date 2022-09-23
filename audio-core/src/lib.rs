//! The core [audio] traits.
//!
//! If you want to build an audio component that is completely agnostic to the
//! shape of any one given audio buffer you can add a dependency directly to
//! these traits instead of depending on all of the [audio] crate.
//!
//! [audio]: https://docs.rs/audio

#![deny(missing_docs, rustdoc::broken_intra_doc_links)]
#![allow(clippy::should_implement_trait)]

pub mod buf;
pub use self::buf::Buf;

mod buf_mut;
pub use self::buf_mut::BufMut;

mod channel;
pub use self::channel::Channel;

mod channel_mut;
pub use self::channel_mut::ChannelMut;

mod translate;
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
