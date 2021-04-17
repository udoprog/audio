//! An idiomatic Rust PipeWire interface.
// Documentation: https://docs.pipewire.org/

mod main_loop;
pub use self::main_loop::MainLoop;

mod property_list;
pub use self::property_list::PropertyList;
