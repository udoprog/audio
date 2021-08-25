macro_rules! slice_comparisons {
    ({$($gen:tt)*}, $a:ty, $b:ty) => {
        impl<$($gen)*> cmp::PartialEq<$b> for $a where T: Copy, T: cmp::PartialEq {
            fn eq(&self, b: &$b) -> bool {
                <[T]>::as_ref(self.buf).eq(<[T]>::as_ref(b))
            }
        }

        impl<$($gen)*> cmp::PartialOrd<$b> for $a where T: Copy, T: cmp::PartialOrd {
            fn partial_cmp(&self, b: &$b) -> Option<cmp::Ordering> {
                <[T]>::as_ref(self.buf).partial_cmp(<[T]>::as_ref(b))
            }
        }
    };
}
