use audio_core::Frame;

/// Accessor trait to use when interacting with a dynamic frame.
trait DynamicSource<T> {}

/// A dynamically sized frame.
pub struct DynamicFrame<'a, T> {
    frame: usize,
    source: &'a dyn DynamicSource<T>,
}

impl<T> Frame for DynamicFrame<'_, T>
where
    T: Copy,
{
    type Sample = T;

    type Frame<'this> = DynamicFrame<'this, Self::Sample>
    where
        Self: 'this;

    type Iter<'this> = DynamicFrameIter<'this, Self::Sample>
    where
        Self: 'this;

    fn as_frame(&self) -> Self::Frame<'_> {
        todo!()
    }

    fn len(&self) -> usize {
        todo!()
    }

    fn get(&self, channel: usize) -> Option<Self::Sample> {
        todo!()
    }

    fn iter(&self) -> Self::Iter<'_> {
        todo!()
    }
}

/// An iterator over a dynamic frame.
pub struct DynamicFrameIter<'a, T> {
    frame: usize,
    source: &'a dyn DynamicSource<T>,
}

impl<T> Iterator for DynamicFrameIter<'_, T>
where
    T: Copy,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

/// An iterator over a set of dynamic frames.
pub struct DynamicIterFrames<'a, T> {
    frame: usize,
    source: &'a dyn DynamicSource<T>,
}

impl<'a, T> Iterator for DynamicIterFrames<'a, T>
where
    T: Copy,
{
    type Item = DynamicFrame<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}
