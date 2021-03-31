//! Reading and writing sequentially from buffers.
//!
//! This is called buffered I/O, and allow buffers to support sequential reading
//! and writing to and from buffer.
//!
//! The primary traits that govern this is [ReadBuf] and [WriteBuf].

pub use rotary_core::{ReadBuf, WriteBuf};

mod utils;
pub use self::utils::{copy, translate};

mod read;
pub use self::read::Read;

mod write;
pub use self::write::Write;

mod read_write;
pub use self::read_write::ReadWrite;
