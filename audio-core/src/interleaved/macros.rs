//! Macros used by interleaved iterators.
//!
// Copied from: https://github.com/rust-lang/rust/blob/master/library/core/src/slice/iter/macros.rs
// Under the MIT license.

// Inlining is_empty and len makes a huge performance difference
macro_rules! is_empty {
    // The way we encode the length of a ZST iterator, this works both for ZST
    // and non-ZST.
    ($self: ident) => {
        $self.ptr.as_ptr() as *const T == $self.end
    };
}

// To get rid of some bounds checks (see `position`), we compute the length in a somewhat
// unexpected way. (Tested by `codegen/slice-position-bounds-check`.)
macro_rules! len {
    ($self: ident) => {{
        let start = $self.ptr;
        let size = size_from_ptr(start.as_ptr());
        let end = $self.end as usize;
        let start = start.as_ptr() as usize;

        if size == 0 {
            // This _cannot_ use `unchecked_sub` because we depend on wrapping
            // to represent the length of long ZST slice iterators.
            end.wrapping_sub(start)
        } else {
            // We know that `start <= end`, so can do better than `offset_from`,
            // which needs to deal in signed.  By setting appropriate flags here
            // we can tell LLVM this, which helps it remove bounds checks.
            // SAFETY: By the type invariant, `start <= end`
            let len = (end - start) / size;

            debug_assert!(
                len % $self.step == 0,
                "len = {}, self.step = {}",
                len,
                $self.step
            );

            len / $self.step
        }
    }};
}

// The shared definition of the `Iter` and `IterMut` iterators
macro_rules! iterator {
    (
        struct $name:ident -> $ptr:ty,
        $elem:ty,
        $raw_mut:tt,
        {$( $mut_:tt )*}
    ) => {
        // Returns the first element and moves the start of the iterator forwards by 1.
        // Greatly improves performance compared to an inlined function. The iterator
        // must not be empty.
        macro_rules! next_unchecked {
            ($self: ident) => {
                $( $mut_ )? *$self.post_inc_start(1)
            }
        }

        // Returns the last element and moves the end of the iterator backwards by 1.
        // Greatly improves performance compared to an inlined function. The iterator
        // must not be empty.
        macro_rules! next_back_unchecked {
            ($self: ident) => {
                $( $mut_ )? *$self.pre_dec_end(1)
            }
        }

        // Shrinks the iterator when T is a ZST, by moving the end of the iterator
        // backwards by `n`. `n` must not exceed `self.len()`.
        macro_rules! zst_shrink {
            ($self: ident, $n: ident) => {
                $self.end = ($self.end as *$raw_mut u8).wrapping_offset(-$n) as *$raw_mut T;
            }
        }

        impl<'a, T> $name<'a, T> {
            // Helper function for moving the start of the iterator forwards by
            // `offset` elements, returning the old start. Unsafe because the
            // offset must not exceed `self.len()`.
            #[inline(always)]
            unsafe fn post_inc_start(&mut self, offset: isize) -> *$raw_mut T {
                if mem::size_of::<T>() == 0 {
                    zst_shrink!(self, offset);
                    self.ptr.as_ptr()
                } else {
                    let offset = offset.saturating_mul(self.step as isize);

                    let old = self.ptr.as_ptr();
                    // SAFETY: the caller guarantees that `offset` doesn't exceed `self.len()`,
                    // so this new pointer is inside `self` and thus guaranteed to be non-null.
                    self.ptr = ptr::NonNull::new_unchecked(self.ptr.as_ptr().wrapping_offset(offset));
                    old
                }
            }

            // Helper function for moving the end of the iterator backwards by
            // `offset` elements, returning the new end. Unsafe because the
            // offset must not exceed `self.len()`.
            #[inline(always)]
            unsafe fn pre_dec_end(&mut self, offset: isize) -> *$raw_mut T {
                if mem::size_of::<T>() == 0 {
                    zst_shrink!(self, offset);
                    self.ptr.as_ptr()
                } else {
                    let offset = offset.saturating_mul(self.step as isize);

                    // SAFETY: the caller guarantees that `offset` doesn't exceed `self.len()`,
                    // which is guaranteed to not overflow an `isize`. Also, the resulting pointer
                    // is in bounds of `slice`, which fulfills the other requirements for `offset`.
                    self.end = self.end.wrapping_offset(-offset);
                    self.end
                }
            }
        }

        impl<'a, T> Iterator for $name<'a, T> where T: Copy {
            type Item = $elem;

            #[inline]
            fn next(&mut self) -> Option<$elem> {
                // could be implemented with slices, but this avoids bounds checks

                // SAFETY: `assume` calls are safe since a slice's start pointer
                // must be non-null, and slices over non-ZSTs must also have a
                // non-null end pointer. The call to `next_unchecked!` is safe
                // since we check if the iterator is empty first.
                unsafe {
                    assert!(!self.ptr.as_ptr().is_null());

                    if mem::size_of::<T>() != 0 {
                        assert!(!self.end.is_null());
                    }

                    if is_empty!(self) {
                        None
                    } else {
                        Some(next_unchecked!(self))
                    }
                }
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                let exact = len!(self);
                (exact, Some(exact))
            }

            #[inline]
            fn count(self) -> usize {
                len!(self)
            }

            #[inline]
            fn nth(&mut self, n: usize) -> Option<$elem> {
                if n >= len!(self) {
                    // This iterator is now empty.
                    if mem::size_of::<T>() == 0 {
                        // We have to do it this way as `ptr` may never be 0, but `end`
                        // could be (due to wrapping).
                        self.end = self.ptr.as_ptr();
                    } else {
                        // SAFETY: end can't be 0 if T isn't ZST because ptr isn't 0 and end >= ptr
                        unsafe {
                            self.ptr = ptr::NonNull::new_unchecked(self.end as *mut T);
                        }
                    }

                    return None;
                }

                // SAFETY: We are in bounds. `post_inc_start` does the right thing even for ZSTs.
                unsafe {
                    self.post_inc_start(n as isize);
                    Some(next_unchecked!(self))
                }
            }
        }

        impl<'a, T> DoubleEndedIterator for $name<'a, T> where T: Copy {
            #[inline]
            fn next_back(&mut self) -> Option<$elem> {
                // could be implemented with slices, but this avoids bounds checks

                // SAFETY: `assume` calls are safe since a slice's start pointer must be non-null,
                // and slices over non-ZSTs must also have a non-null end pointer.
                // The call to `next_back_unchecked!` is safe since we check if the iterator is
                // empty first.
                unsafe {
                    assert!(!self.ptr.as_ptr().is_null());

                    if mem::size_of::<T>() != 0 {
                        assert!(!self.end.is_null());
                    }

                    if is_empty!(self) {
                        None
                    } else {
                        Some(next_back_unchecked!(self))
                    }
                }
            }

            #[inline]
            fn nth_back(&mut self, n: usize) -> Option<$elem> {
                if n >= len!(self) {
                    // This iterator is now empty.
                    self.end = self.ptr.as_ptr();
                    return None;
                }

                // SAFETY: We are in bounds. `pre_dec_end` does the right thing even for ZSTs.
                unsafe {
                    self.pre_dec_end(n as isize);
                    Some(next_back_unchecked!(self))
                }
            }
        }
    }
}

macro_rules! interleaved_channel {
    ($lt:lifetime, $arg:ident, $raw_mut:tt, $name:ident) => {
        macro_rules! zst_set_len {
            ($self:ident, $n:ident) => {
                $self.end = ($self.ptr.as_ptr() as *$raw_mut u8).wrapping_add($n) as *$raw_mut T;
            }
        }

        impl<$lt, $arg> $name<$lt, $arg> {
            /// Construct an interleaved channel buffer.
            ///
            /// The provided buffer must be the complete buffer, which includes *all*
            /// other channels. The provided `channels` argument is the total number of
            /// channels in this buffer, and `channel` indicates which specific channel
            /// this buffer belongs to.
            ///
            /// Note that this is typically not used directly, but instead through an
            /// abstraction which makes sure to provide the correct parameters.
            ///
            /// # Safety
            ///
            /// Caller must ensure that the provided base pointer, length, and
            /// channel configuration is in bounds with the buffer pointed to by
            /// `ptr`.
            pub unsafe fn new_unchecked(
                ptr: ptr::NonNull<$arg>,
                len: usize,
                channel: usize,
                channels: usize,
            ) -> Self {
                debug_assert!(
                    channel < channels,
                    "referencing channel out of bounds; channel={}, channels={}",
                    channel,
                    channels,
                );
                debug_assert!(
                    len % channels == 0,
                    "number of channels misaligned with length; channels={}, len={}",
                    channels,
                    len,
                );
                debug_assert!(
                    channels <= len,
                    "number of channels out of bounds; channels={}, len={}",
                    channels,
                    len,
                );

                let ptr = ptr.as_ptr();

                let (ptr, end) = if mem::size_of::<T>() == 0 {
                    let end = (ptr as *$raw_mut u8).wrapping_add(len / channels) as *$raw_mut $arg;
                    (ptr, end)
                } else {
                    let ptr = ptr.add(channel);
                    let end = ptr.wrapping_add(len) as *$raw_mut $arg;
                    (ptr, end)
                };

                Self {
                    ptr: ptr::NonNull::new_unchecked(ptr),
                    end,
                    step: channels,
                    _marker: marker::PhantomData,
                }
            }
        }

        impl<$lt, $arg> $name<$lt, $arg> where $arg: Copy {
            /// Get the given frame if it's in bound.
            pub fn get(&self, frame: usize) -> Option<T> {
                if frame < len!(self) {
                    if mem::size_of::<T>() == 0 {
                        Some(unsafe { *(self.ptr.as_ptr() as *const _) })
                    } else {
                        let add = frame.saturating_mul(self.step);
                        Some(unsafe { *(self.ptr.as_ptr() as *const $arg).add(add) })
                    }
                } else {
                    None
                }
            }

            /// Construct an iterator over the interleaved channel.
            pub fn iter(&self) -> Iter<'_, $arg> {
                Iter {
                    ptr: self.ptr,
                    end: self.end,
                    step: self.step,
                    _marker: marker::PhantomData,
                }
            }
        }

        impl<$lt, $arg> Channel for $name<$lt, $arg>
        where
            $arg: Copy,
        {
            type Sample = $arg;

            type Iter<'s>
            where
                $arg: 's,
            = Iter<'s, $arg>;

            fn frames(&self) -> usize {
                len!(self)
            }

            fn iter(&self) -> Self::Iter<'_> {
                (*self).iter()
            }

            fn skip(mut self, n: usize) -> Self {
                if mem::size_of::<T>() == 0 {
                    let len = len!(self).saturating_sub(n);
                    zst_set_len!(self, len);
                } else {
                    let len = usize::min(len!(self), n).saturating_mul(self.step);
                    // Safety: internal invariants in this structure ensures it
                    // doesn't go out of bounds.
                    self.ptr = unsafe { ptr::NonNull::new_unchecked(self.ptr.as_ptr().wrapping_add(len)) };
                }

                self
            }

            fn tail(mut self, n: usize) -> Self {
                if mem::size_of::<T>() == 0 {
                    let len = usize::min(len!(self), n);
                    zst_set_len!(self, len);
                } else {
                    let offset = len!(self).saturating_sub(n).saturating_mul(self.step);
                    // Safety: internal invariants in this structure ensures it
                    // doesn't go out of bounds.
                    self.ptr = unsafe { ptr::NonNull::new_unchecked(self.ptr.as_ptr().wrapping_add(offset)) };
                }

                self
            }

            fn limit(mut self, limit: usize) -> Self {
                if mem::size_of::<T>() == 0 {
                    let len = usize::min(len!(self), limit);
                    zst_set_len!(self, len);
                } else {
                    let len = len!(self).saturating_sub(limit).saturating_mul(self.step);
                    // Safety: internal invariants in this structure ensures it
                    // doesn't go out of bounds.
                    self.end = self.end.wrapping_sub(len);
                }

                self
            }

            fn chunk(mut self, n: usize, window: usize) -> Self {
                let n = n.saturating_mul(window);
                let len = len!(self);

                if mem::size_of::<T>() == 0 {
                    let len = usize::min(len.saturating_sub(n), window);
                    zst_set_len!(self, len);
                } else {
                    let ptr = usize::min(len, n).saturating_mul(self.step);
                    let end = len.saturating_sub(n).saturating_sub(window).saturating_mul(self.step);

                    // Safety: internal invariants in this structure ensures it
                    // doesn't go out of bounds.
                    unsafe {
                        self.ptr = ptr::NonNull::new_unchecked(self.ptr.as_ptr().wrapping_add(ptr));
                        self.end = self.end.wrapping_sub(end);
                    };
                }

                self
            }

            fn as_linear(&self) -> Option<&[T]> {
                None
            }
        }
    };
}

macro_rules! comparisons {
    ({$($gen:tt)*}, $a:ty, $b:ty) => {
        impl<$($gen)*> fmt::Debug for $a where T: Copy + fmt::Debug {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_list().entries(self.iter()).finish()
            }
        }

        impl<$($gen)*> cmp::PartialEq<$a> for $a where T: Copy + cmp::PartialEq {
            fn eq(&self, other: &$a) -> bool {
                self.iter().eq(other.iter())
            }
        }

        impl<$($gen)*> cmp::PartialEq<$b> for $a where T: Copy + cmp::PartialEq {
            fn eq(&self, other: &$b) -> bool {
                self.iter().eq(other.iter())
            }
        }

        impl<$($gen)*> cmp::Eq for $a where T: Copy + cmp::Eq {
        }

        impl<$($gen)*> cmp::PartialOrd<$a> for $a where T: Copy + cmp::PartialOrd {
            fn partial_cmp(&self, other: &$a) -> Option<cmp::Ordering> {
                self.iter().partial_cmp(other.iter())
            }
        }

        impl<$($gen)*> cmp::PartialOrd<$b> for $a where T: Copy + cmp::PartialOrd {
            fn partial_cmp(&self, other: &$b) -> Option<cmp::Ordering> {
                self.iter().partial_cmp(other.iter())
            }
        }

        impl<$($gen)*> cmp::Ord for $a where T: Copy + cmp::Ord {
            fn cmp(&self, other: &$a) -> cmp::Ordering {
                self.iter().cmp(other.iter())
            }
        }
    };
}

macro_rules! slice_comparisons {
    ({$($gen:tt)*}, $a:ty, $b:ty) => {
        impl<$($gen)*> cmp::PartialEq<$b> for $a where T: Copy, T: cmp::PartialEq {
            fn eq(&self, b: &$b) -> bool {
                (*self).iter().eq(b.iter().copied())
            }
        }

        impl<$($gen)*> cmp::PartialOrd<$b> for $a where T: Copy, T: cmp::PartialOrd {
            fn partial_cmp(&self, b: &$b) -> Option<cmp::Ordering> {
                (*self).iter().partial_cmp(b.iter().copied())
            }
        }
    };
}
