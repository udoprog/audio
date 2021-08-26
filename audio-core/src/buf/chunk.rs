use crate::{Buf, BufMut, Channel, ChannelMut, ExactSizeBuf};

/// A chunk of another buffer.
///
/// See [Buf::chunk].
pub struct Chunk<B> {
    buf: B,
    n: usize,
    window: usize,
}

impl<B> Chunk<B> {
    /// Construct a new limited buffer.
    pub(crate) fn new(buf: B, n: usize, window: usize) -> Self {
        Self { buf, n, window }
    }
}

/// ```rust
/// use audio::Buf;
///
/// let buf = audio::interleaved![[0; 4]; 2];
///
/// assert_eq!((&buf).chunk(0, 3).channels(), 2);
/// assert_eq!((&buf).chunk(0, 3).frames_hint(), Some(3));
///
/// assert_eq!((&buf).chunk(1, 3).channels(), 2);
/// assert_eq!((&buf).chunk(1, 3).frames_hint(), Some(1));
///
/// assert_eq!((&buf).chunk(2, 3).channels(), 2);
/// assert_eq!((&buf).chunk(2, 3).frames_hint(), Some(0));
/// ```
impl<B> Buf for Chunk<B>
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
        let len = frames.saturating_sub(self.n.saturating_mul(self.window));
        Some(usize::min(len, self.window))
    }

    fn channels(&self) -> usize {
        self.buf.channels()
    }

    fn get(&self, channel: usize) -> Option<Self::Channel<'_>> {
        Some(self.buf.get(channel)?.chunk(self.n, self.window))
    }

    fn iter(&self) -> Self::Iter<'_> {
        Iter {
            iter: self.buf.iter(),
            n: self.n,
            window: self.window,
        }
    }
}

impl<B> BufMut for Chunk<B>
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
        Some(self.buf.get_mut(channel)?.chunk(self.n, self.window))
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
            window: self.window,
        }
    }
}

/// ```rust
/// use audio::{Buf, ExactSizeBuf};
///
/// let buf = audio::interleaved![[0; 4]; 2];
///
/// assert_eq!((&buf).chunk(0, 3).frames(), 3);
/// assert_eq!((&buf).chunk(1, 3).frames(), 1);
/// assert_eq!((&buf).chunk(2, 3).frames(), 0);
/// ```
impl<B> ExactSizeBuf for Chunk<B>
where
    B: ExactSizeBuf,
{
    fn frames(&self) -> usize {
        let len = self
            .buf
            .frames()
            .saturating_sub(self.n.saturating_mul(self.window));
        usize::min(len, self.window)
    }
}

iterators!(n: usize, window: usize => self.chunk(n, window));
