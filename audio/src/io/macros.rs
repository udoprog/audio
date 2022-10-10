macro_rules! iter {
    (
        $ident:ident,
        $($field:ident : $field_ty:ty),* $(,)?
        =>
        $self:ident $(. $fn:ident ($($arg:ident),* $(,)?))*
    ) => {
        pub struct $ident<'a, B>
        where
            B: 'a + Buf,
        {
            iter: B::IterChannels<'a>,
            $($field: $field_ty,)*
        }

        impl<'a, B> Iterator for $ident<'a, B>
        where
            B: 'a + Buf,
        {
            type Item = B::Channel<'a>;

            fn next(&mut $self) -> Option<Self::Item> {
                let channel = $self.iter.next()?;
                Some(channel $(. $fn ($($self . $arg),*))*)
            }
        }
    }
}

macro_rules! iter_mut {
    (
        $ident:ident,
        $($field:ident : $field_ty:ty),* $(,)?
        =>
        $self:ident $(. $fn:ident ($($arg:ident),* $(,)?))*
    ) => {
        pub struct $ident<'a, B>
        where
            B: 'a + BufMut,
        {
            iter: B::IterChannelsMut<'a>,
            $($field: $field_ty,)*
        }

        impl<'a, B> Iterator for $ident<'a, B>
        where
            B: 'a + BufMut,
        {
            type Item = B::ChannelMut<'a>;

            fn next(&mut $self) -> Option<Self::Item> {
                let channel = $self.iter.next()?;
                Some(channel $(. $fn ($($self . $arg),*))*)
            }
        }
    }
}
