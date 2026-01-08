#![allow(unused)]

macro_rules! cfg_events_driver {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "events-driver")]
            #[cfg_attr(docsrs, doc(
                cfg(feature = "events-driver")
            ))]
            $item
        )*
    }
}

macro_rules! cfg_poll_driver {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "poll-driver")]
            #[cfg_attr(docsrs, doc(
                cfg(feature = "poll-driver")
            ))]
            $item
        )*
    }
}
