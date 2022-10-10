//! Common helpers for frame abstractions.

mod interleaved;
pub(crate) use self::interleaved::RawInterleaved;
pub use self::interleaved::{InterleavedFrame, InterleavedFramesIter};

mod sequential;
pub(crate) use self::sequential::RawSequential;
pub use self::sequential::{SequentialFrame, SequentialFramesIter};
