//! Unix-specific types and definitions.

pub mod errno;
pub mod poll;
#[doc(inline)]
pub use nix::Error;

macro_rules! errno {
    ($expr:expr) => {{
        let result = $expr;

        if result < 0 {
            Err($crate::unix::errno::Errno::from_i32(-result as i32))
        } else {
            Ok(result)
        }
    }};
}
