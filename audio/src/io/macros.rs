macro_rules! iter {
    (
        $($field:ident : $field_ty:ty),* $(,)?
        =>
        $self:ident $(. $fn:ident ($($arg:ident),* $(,)?))*
    ) => {
        pub struct Iter<'a, B>
        where
            B: 'a + Buf,
        {
            iter: B::IterChannels<'a>,
            $($field: $field_ty,)*
        }

        impl<'a, B> Iterator for Iter<'a, B>
        where
            B: 'a + Buf,
        {
            type Item = B::Channel<'a>;

            #[inline]
            fn next(&mut $self) -> Option<Self::Item> {
                let channel = $self.iter.next()?;
                Some(channel $(. $fn ($($self . $arg),*))*)
            }
        }
    }
}

macro_rules! iter_mut {
    (
        $($field:ident : $field_ty:ty),* $(,)?
        =>
        $self:ident $(. $fn:ident ($($arg:ident),* $(,)?))*
    ) => {
        pub struct IterMut<'a, B>
        where
            B: 'a + BufMut,
        {
            iter: B::IterChannelsMut<'a>,
            $($field: $field_ty,)*
        }

        impl<'a, B> Iterator for IterMut<'a, B>
        where
            B: 'a + BufMut,
        {
            type Item = B::ChannelMut<'a>;

            #[inline]
            fn next(&mut $self) -> Option<Self::Item> {
                let channel = $self.iter.next()?;
                Some(channel $(. $fn ($($self . $arg),*))*)
            }
        }
    }
}
