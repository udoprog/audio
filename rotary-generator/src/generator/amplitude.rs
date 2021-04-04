use crate::generator::Generator;
use rotary_core::Translate;
use std::ops;

/// A generator combinator that adjusts the amplitude of the generated value.
///
/// See [Generator::amplitude].
pub struct Amplitude<G> {
    generator: G,
    amplitude: f32,
}

impl<G> Amplitude<G> {
    pub(super) fn new(generator: G, amplitude: f32) -> Self {
        Self {
            generator,
            amplitude,
        }
    }
}

impl<G> Generator for Amplitude<G>
where
    G: Generator,
    G::Sample: Translate<f32>,
    G::Sample: ops::Mul<Output = G::Sample>,
{
    type Sample = G::Sample;

    fn sample(&mut self) -> G::Sample {
        self.generator.sample() * Self::Sample::translate(self.amplitude)
    }
}
