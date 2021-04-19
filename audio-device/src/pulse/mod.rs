//! An idiomatic Rust PulseAudio interface.
// Documentation: https://freedesktop.org/software/pulseaudio/doxygen/
// Ref: https://gist.github.com/toroidal-code/8798775

#[macro_use]
mod error;
pub use self::error::{Error, Result};

mod enums;
pub use self::enums::ContextState;

mod main_loop;
pub use self::main_loop::MainLoop;

mod property_list;
pub use self::property_list::PropertyList;

mod context;
pub use self::context::Context;
