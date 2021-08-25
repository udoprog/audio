macro_rules! iterators {
    (
        $($field:ident : $field_ty:ty),* $(,)?
        =>
        $self:ident . $fn:ident ($($arg:ident),* $(,)?)
    ) => {
        pub struct Iter<I> {
            iter: I,
            $($field: $field_ty,)*
        }

        impl<I> Iterator for Iter<I>
        where
            I: Iterator,
            I::Item: Channel,
        {
            type Item = I::Item;

            fn next(&mut $self) -> Option<Self::Item> {
                Some($self.iter.next()?.$fn($($self.$arg),*))
            }
        }

        pub struct IterMut<I> {
            iter: I,
            $($field: $field_ty,)*
        }

        impl<I> Iterator for IterMut<I>
        where
            I: Iterator,
            I::Item: ChannelMut,
        {
            type Item = I::Item;

            fn next(&mut $self) -> Option<Self::Item> {
                Some($self.iter.next()?.$fn($($self . $arg),*))
            }
        }
    }
}
