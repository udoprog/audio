use crate::mask::Mask;
/// A mask where every element is set.
#[derive(Default, Debug, Clone, Copy)]
pub struct All(());

impl Mask for All {
    type Iter = Iter;

    fn test(&self, _: usize) -> bool {
        true
    }

    fn iter(&self) -> Self::Iter {
        Iter { index: 0 }
    }
}

/// The iterator for the [All] mask. Yields every possible index in order.
pub struct Iter {
    index: usize,
}

impl Iterator for Iter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.index;
        self.index += 1;
        Some(index)
    }
}
