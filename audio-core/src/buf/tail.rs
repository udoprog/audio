use crate::{Buf, BufMut, Channel, ChannelMut, ExactSizeBuf, ReadBuf};

/// The tail of a buffer.
///
/// See [Buf::tail].
pub struct Tail<B> {
    buf: B,
    n: usize,
}

impl<B> Tail<B> {
    /// Construct a new buffer tail.
    pub(crate) fn new(buf: B, n: usize) -> Self {
        Self { buf, n }
    }
}

impl<B> Buf for Tail<B>
where
    B: Buf,
{
    type Sample = B::Sample;

    type Channel<'a>
    where
        Self::Sample: 'a,
    = B::Channel<'a>;

    type Iter<'a>
    where
        Self::Sample: 'a,
    = Iter<B::Iter<'a>>;

    fn frames_hint(&self) -> Option<usize> {
        let frames = self.buf.frames_hint()?;
        Some(usize::min(frames, self.n))
    }

    fn channels(&self) -> usize {
        self.buf.channels()
    }

    fn get(&self, channel: usize) -> Option<Self::Channel<'_>> {
        Some(self.buf.get(channel)?.tail(self.n))
    }

    fn iter(&self) -> Self::Iter<'_> {
        Iter {
            iter: self.buf.iter(),
            n: self.n,
        }
    }
}

impl<B> BufMut for Tail<B>
where
    B: BufMut,
{
    type ChannelMut<'a>
    where
        Self::Sample: 'a,
    = B::ChannelMut<'a>;

    type IterMut<'a>
    where
        Self::Sample: 'a,
    = IterMut<B::IterMut<'a>>;

    fn get_mut(&mut self, channel: usize) -> Option<Self::ChannelMut<'_>> {
        Some(self.buf.get_mut(channel)?.tail(self.n))
    }

    fn copy_channels(&mut self, from: usize, to: usize)
    where
        Self::Sample: Copy,
    {
        self.buf.copy_channels(from, to);
    }

    fn iter_mut(&mut self) -> Self::IterMut<'_> {
        IterMut {
            iter: self.buf.iter_mut(),
            n: self.n,
        }
    }
}

/// [Tail] adjusts the implementation of [ExactSizeBuf].
///
/// ```
/// use audio::{Buf, ExactSizeBuf};
///
/// let buf = audio::interleaved![[0; 4]; 2];
///
/// assert_eq!((&buf).tail(0).frames(), 0);
/// assert_eq!((&buf).tail(1).frames(), 1);
/// assert_eq!((&buf).tail(5).frames(), 4);
/// ```
impl<B> ExactSizeBuf for Tail<B>
where
    B: ExactSizeBuf,
{
    fn frames(&self) -> usize {
        usize::min(self.buf.frames(), self.n)
    }
}

impl<B> ReadBuf for Tail<B>
where
    B: ReadBuf,
{
    fn remaining(&self) -> usize {
        usize::min(self.buf.remaining(), self.n)
    }

    fn advance(&mut self, n: usize) {
        let n = self
            .buf
            .remaining()
            .saturating_sub(self.n)
            .saturating_add(n);

        self.buf.advance(n);
    }
}

iterators!(n: usize => self.tail(n));
