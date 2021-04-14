/// Macro to use for modules constrained to any windows-specific audio drivers.
macro_rules! cfg_any_windows {
    ($($item:item)*) => {
        $(
            #[cfg(any(feature = "wasapi", feature = "events-driver"))]
            #[cfg_attr(docsrs, doc(
                cfg(any(feature = "wasapi", feature = "events-driver")))
            )]
            $item
        )*
    }
}

/// Macro to use for modules constrained to any unix-specific audio drivers.
macro_rules! cfg_any_unix {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "alsa")]
            #[cfg_attr(docsrs, doc(
                cfg(feature = "alsa")
            ))]
            $item
        )*
    }
}

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

macro_rules! cfg_wasapi {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "wasapi")]
            #[cfg_attr(docsrs, doc(
                cfg(feature = "wasapi")
            ))]
            $item
        )*
    }
}

macro_rules! cfg_alsa {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "alsa")]
            #[cfg_attr(docsrs, doc(
                cfg(feature = "alsa")
            ))]
            $item
        )*
    }
}
