use crate::generator::Generator;

/// An iterator constructed from a [Generator].
///
/// See [Generator::iter].
pub struct Iter<G> {
    generator: G,
}

impl<G> Iter<G> {
    pub(super) fn new(generator: G) -> Self {
        Self { generator }
    }
}

impl<G> Iterator for Iter<G>
where
    G: Generator,
{
    type Item = G::Sample;

    fn next(&mut self) -> Option<Self::Item> {
        Some(self.generator.sample())
    }
}
