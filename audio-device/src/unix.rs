//! Unix-specific types and definitions.

pub mod errno;
pub mod poll;
#[doc(inline)]
pub use nix::Error;
