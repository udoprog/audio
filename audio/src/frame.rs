//! Common helpers for frame abstractions.

mod dynamic;
pub use self::dynamic::{DynamicFrame, DynamicFrameIter, DynamicIterFrames};

mod interleaved;
pub(crate) use self::interleaved::RawInterleaved;
pub use self::interleaved::{InterleavedFrame, InterleavedIterFrames};

mod sequential;
pub(crate) use self::sequential::RawSequential;
pub use self::sequential::{SequentialFrame, SequentialIterFrames};
