//! Audio generators.
//!
//! This provides audio generators which implements the [Generator] trait.
//!
//! It is part of the [audio ecosystem] of crates.
//!
//! # Examples
//!
//! ```rust
//! use audio_generator::{Generator, Sine};
//!
//! let mut g = Sine::new(440.0, 44100.0);
//! assert_eq!(g.sample(), 0.0);
//! assert!(g.sample() > 0.0);
//! ```
//!
//! [audio ecosystem]: https://docs.rs/audio

mod generator;
pub use self::generator::Generator;

mod sine;
pub use self::sine::Sine;
