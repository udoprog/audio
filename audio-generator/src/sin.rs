use crate::generator::Generator;

/// A sine tone generator.
///
/// # Examples
///
/// ```rust
/// use rotary_generator::{Generator, Sin};
///
/// let mut g = Sin::new(440.0, 44100.0);
/// assert_eq!(g.sample(), 0.0);
/// assert!(g.sample() > 0.0);
/// ```
pub struct Sin {
    at: f32,
    step: f32,
    round_at: f32,
}

impl Sin {
    /// Construct a new sine tone generator. The generated tone has the given
    /// `rate` adjusted for the provided `sample_rate`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rotary_generator::{Generator, Sin};
    ///
    /// let mut g = Sin::new(440.0, 44100.0);
    /// assert_eq!(g.sample(), 0.0);
    /// assert!(g.sample() > 0.0);
    /// ```
    pub fn new(rate: f32, sample_rate: f32) -> Self {
        let freq = rate / sample_rate / 2.0;
        let step = 2.0 * std::f32::consts::PI * freq;

        Self {
            at: 0.0,
            step,
            round_at: step / freq,
        }
    }
}

impl Iterator for Sin {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.sample())
    }
}

impl Generator for Sin {
    type Sample = f32;

    fn sample(&mut self) -> Self::Sample {
        let f = self.at;
        self.at += self.step;

        if self.at > self.round_at {
            self.at -= self.round_at;
        }

        f
    }
}
