//! The core traits for [rotary].
//!
//! If you want to build an audio component that is completely agnostic to the
//! shape of any one given audio buffer you can add a dependency directly to
//! these traits instead of depending on all of rotary.
//!
//! [rotary]: https://github.com/udoprog/rotary

#![deny(missing_docs, broken_intra_doc_links)]
#![allow(clippy::should_implement_trait)]

mod buf;
pub use self::buf::{
    AsInterleaved, AsInterleavedMut, Buf, Channels, ChannelsMut, ExactSizeBuf, InterleavedBuf,
    ResizableBuf,
};

mod channel;
pub use self::channel::{Channel, ChannelMut};

mod translate;
pub use self::translate::Translate;

mod sample;
pub use self::sample::Sample;

mod io;
pub use self::io::{ReadBuf, WriteBuf};
