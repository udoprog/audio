mod amplitude;
pub use self::amplitude::Amplitude;

mod iter;
pub use self::iter::Iter;

/// The trait for an audio generator.
pub trait Generator {
    /// The sample that is generated.
    type Sample;

    /// Generate the next sample from the generator. Advances the underlying
    /// generator to the next sample.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio_generator::{Generator, Sin};
    ///
    /// let mut g = Sin::new(440.0, 44100.0);
    /// assert_eq!(g.sample(), 0.0);
    /// assert!(g.sample() > 0.0);
    /// ```
    fn sample(&mut self) -> Self::Sample;

    /// Construct an iterator from this generator.
    ///
    /// ```rust
    /// use audio_generator::{Generator, Sin};
    ///
    /// let mut g = Sin::new(440.0, 44100.0).amplitude(0.5);
    /// let samples = g.iter().take(10).collect::<Vec<f32>>();
    ///
    /// assert_eq!(samples.len(), 10);
    /// ```
    fn iter(self) -> Iter<Self>
    where
        Self: Sized,
    {
        Iter::new(self)
    }

    /// Modify the amplitude of the sample.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use audio_generator::{Generator, Sin};
    ///
    /// let mut a = Sin::new(440.0, 44100.0).amplitude(0.1);
    /// let mut b = Sin::new(440.0, 44100.0).amplitude(0.1);
    ///
    /// for _ in 0..100 {
    ///     assert!(a.sample().abs() <= b.sample().abs());
    /// }
    /// ```
    fn amplitude(self, amplitude: f32) -> Amplitude<Self>
    where
        Self: Sized,
    {
        Amplitude::new(self, amplitude)
    }
}
