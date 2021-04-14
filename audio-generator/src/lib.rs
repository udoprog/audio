//! Audio generators.
//!
//! This provides audio generators which implements the [Generator] trait.
//!
//! It is part of the [audio ecosystem] of crates.
//!
//! [audio ecosystem]: https://docs.rs/audio

mod generator;
pub use self::generator::Generator;

mod sine;
pub use self::sine::Sine;
