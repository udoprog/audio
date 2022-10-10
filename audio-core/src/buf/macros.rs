macro_rules! iterators {
    (
        $ident:ident, $ident_mut:ident,
        $($field:ident : $field_ty:ty),* $(,)?
        =>
        $self:ident . $fn:ident ($($arg:ident),* $(,)?)
    ) => {
        pub struct $ident<I> {
            iter: I,
            $($field: $field_ty,)*
        }

        impl<I> Iterator for $ident<I>
        where
            I: Iterator,
            I::Item: Channel,
        {
            type Item = I::Item;

            fn next(&mut $self) -> Option<Self::Item> {
                Some($self.iter.next()?.$fn($($self.$arg),*))
            }

            fn nth(&mut $self, n: usize) -> Option<Self::Item> {
                Some($self.iter.nth(n)?.$fn($($self.$arg),*))
            }
        }

        impl<I> DoubleEndedIterator for $ident<I>
        where
            I: DoubleEndedIterator,
            I::Item: Channel,
        {
            fn next_back(&mut $self) -> Option<Self::Item> {
                Some($self.iter.next_back()?.$fn($($self.$arg),*))
            }

            fn nth_back(&mut $self, n: usize) -> Option<Self::Item> {
                Some($self.iter.nth_back(n)?.$fn($($self.$arg),*))
            }
        }

        impl<I> ExactSizeIterator for $ident<I>
        where
            I: ExactSizeIterator,
            I::Item: ChannelMut,
        {
            fn len(&$self) -> usize {
                $self.iter.len()
            }
        }

        pub struct $ident_mut<I> {
            iter: I,
            $($field: $field_ty,)*
        }

        impl<I> Iterator for $ident_mut<I>
        where
            I: Iterator,
            I::Item: ChannelMut,
        {
            type Item = I::Item;

            fn next(&mut $self) -> Option<Self::Item> {
                Some($self.iter.next()?.$fn($($self . $arg),*))
            }

            fn nth(&mut $self, n: usize) -> Option<Self::Item> {
                Some($self.iter.nth(n)?.$fn($($self . $arg),*))
            }
        }

        impl<I> DoubleEndedIterator for $ident_mut<I>
        where
            I: DoubleEndedIterator,
            I::Item: ChannelMut,
        {
            fn next_back(&mut $self) -> Option<Self::Item> {
                Some($self.iter.next_back()?.$fn($($self . $arg),*))
            }

            fn nth_back(&mut $self, n: usize) -> Option<Self::Item> {
                Some($self.iter.nth_back(n)?.$fn($($self . $arg),*))
            }
        }

        impl<I> ExactSizeIterator for $ident_mut<I>
        where
            I: ExactSizeIterator,
            I::Item: ChannelMut,
        {
            fn len(&$self) -> usize {
                $self.iter.len()
            }
        }
    }
}
