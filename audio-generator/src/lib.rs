//! Audio generators.
//!
//! Central to these is the [Generator] trait which allows for describing an
//! abstract generator in an object-safe manner.

mod generator;
pub use self::generator::Generator;

mod sin;
pub use self::sin::Sin;
