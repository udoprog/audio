use crate::{Buf, BufMut, Channel, ChannelMut, ExactSizeBuf, ReadBuf};

use core::ops;
use core::ops::Bound;

/// A range of a buffer
///
/// See [Buf::range].
pub struct Range<B> {
    buf: B,
    range: ops::Range<usize>,
}

impl<B> Range<B> {
    /// Construct a new limited buffer.
    pub(crate) fn new(buf: B, range: impl ops::RangeBounds<usize>) -> Self
    where B: Buf {
        let start = match range.start_bound() {
            Bound::Unbounded => 0,
            Bound::Included(&i) => i,
            Bound::Excluded(&i) => i + 1,
        };
        let max = buf.frames_hint().expect("Unable to check bounds of range for Buf because frames_hint returned None");
        let end = match range.end_bound() {
            Bound::Unbounded => max,
            Bound::Included(&i) => i + 1,
            Bound::Excluded(&i) => i,
        };
        assert!(end <= max, "End index {} out of bounds, maximum {}", end, max);
        assert!(start <= end);
        let range = core::ops::Range{ start, end };
        Self { buf, range }
    }
}

impl<B> Buf for Range<B>
where
    B: Buf,
{
    type Sample = B::Sample;

    type Channel<'this> = B::Channel<'this>
    where
        Self: 'this;

    type IterChannels<'this> = IterChannels<B::IterChannels<'this>>
    where
        Self: 'this;

    fn frames_hint(&self) -> Option<usize> {
        Some(self.range.len())
    }

    fn channels(&self) -> usize {
        self.buf.channels()
    }

    fn get_channel(&self, channel: usize) -> Option<Self::Channel<'_>> {
        Some(self.buf.get_channel(channel)?.range(self.range.clone()))
    }

    fn iter_channels(&self) -> Self::IterChannels<'_> {
        IterChannels {
            iter: self.buf.iter_channels(),
            range: self.range.clone(),
        }
    }
}

impl<B> BufMut for Range<B>
where
    B: BufMut,
{
    type ChannelMut<'this> = B::ChannelMut<'this>
    where
        Self: 'this;

    type IterChannelsMut<'this> = IterChannelsMut<B::IterChannelsMut<'this>>
    where
        Self: 'this;

    fn get_channel_mut(&mut self, channel: usize) -> Option<Self::ChannelMut<'_>> {
        Some(self.buf.get_channel_mut(channel)?.range(self.range.clone()))
    }

    fn copy_channel(&mut self, from: usize, to: usize)
    where
        Self::Sample: Copy,
    {
        self.buf.copy_channel(from, to);
    }

    fn iter_channels_mut(&mut self) -> Self::IterChannelsMut<'_> {
        IterChannelsMut {
            iter: self.buf.iter_channels_mut(),
            range: self.range.clone(),
        }
    }
}

/// [Range] adjusts the implementation of [ExactSizeBuf] to take the frame
/// limiting into account.
///
/// ```
/// use audio::{Buf, ExactSizeBuf};
///
/// let buf = audio::interleaved![[0; 4]; 2];
///
/// assert_eq!((&buf).limit(0).frames(), 0);
/// assert_eq!((&buf).limit(1).frames(), 1);
/// assert_eq!((&buf).limit(5).frames(), 4);
/// ```
impl<B> ExactSizeBuf for Range<B>
where
    B: ExactSizeBuf,
{
    fn frames(&self) -> usize {
        self.range.len()
    }
}

impl<B> ReadBuf for Range<B>
where
    B: ReadBuf,
{
    fn remaining(&self) -> usize {
        usize::min(self.buf.remaining(), self.range.len())
    }

    fn advance(&mut self, n: usize) {
        self.buf.advance(usize::min(n, self.range.len()));
    }
}

// TODO: fix macro
// iterators!(range: core::ops::Range<usize> => self.range(range.clone(());

pub struct IterChannels<I> {
    iter: I,
    range: core::ops::Range<usize>,
}
impl<I> Iterator for IterChannels<I>
where
    I: Iterator,
    I::Item: Channel,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.iter.next()?.range(self.range.clone()))
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        Some(self.iter.nth(n)?.range(self.range.clone()))
    }
}
impl<I> DoubleEndedIterator for IterChannels<I>
where
    I: DoubleEndedIterator,
    I::Item: Channel,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        Some(self.iter.next_back()?.range(self.range.clone()))
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        Some(self.iter.nth_back(n)?.range(self.range.clone()))
    }
}
impl<I> ExactSizeIterator for IterChannels<I>
where
    I: ExactSizeIterator,
    I::Item: ChannelMut,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}
pub struct IterChannelsMut<I> {
    iter: I,
    range: core::ops::Range<usize>,
}
impl<I> Iterator for IterChannelsMut<I>
where
    I: Iterator,
    I::Item: ChannelMut,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        Some(self.iter.next()?.range(self.range.clone()))
    }
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        Some(self.iter.nth(n)?.range(self.range.clone()))
    }
}
impl<I> DoubleEndedIterator for IterChannelsMut<I>
where
    I: DoubleEndedIterator,
    I::Item: ChannelMut,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        Some(self.iter.next_back()?.range(self.range.clone()))
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        Some(self.iter.nth_back(n)?.range(self.range.clone()))
    }
}
impl<I> ExactSizeIterator for IterChannelsMut<I>
where
    I: ExactSizeIterator,
    I::Item: ChannelMut,
{
    fn len(&self) -> usize {
        self.iter.len()
    }
}
