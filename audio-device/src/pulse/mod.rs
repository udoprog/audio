//! An idiomatic Rust PulseAudio interface.
// Documentation: https://freedesktop.org/software/pulseaudio/doxygen/
// Ref: https://gist.github.com/toroidal-code/8798775

mod main_loop;
pub use self::main_loop::MainLoop;

mod property_list;
pub use self::property_list::PropertyList;
