use crate::mask::Mask;
/// A mask where every element is set.
#[derive(Default, Debug, Clone, Copy)]
pub struct None(());

impl Mask for None {
    type Iter = Iter;

    fn test(&self, _: usize) -> bool {
        false
    }

    fn iter(&self) -> Self::Iter {
        Iter(())
    }
}

/// The iterator for the [None] mask. Yields every possible index in order.
pub struct Iter(());

impl Iterator for Iter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        Option::None
    }
}
