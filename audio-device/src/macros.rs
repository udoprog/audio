#![allow(unused)]

/// Macro to use for modules constrained to any windows-specific audio drivers.
macro_rules! cfg_windows {
    ($($item:item)*) => {
        $(
            #[cfg(any(feature = "windows"))]
            #[cfg_attr(docsrs, doc(
                cfg(any(feature = "windows")))
            )]
            $item
        )*
    }
}

/// Macro to use for modules constrained to any unix-specific audio drivers.
macro_rules! cfg_unix {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "unix")]
            #[cfg_attr(docsrs, doc(
                cfg(feature = "unix")
            ))]
            $item
        )*
    }
}

macro_rules! cfg_libc {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "libc")]
            #[cfg_attr(docsrs, doc(
                cfg(feature = "libc")
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

macro_rules! cfg_pulse {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "pulse")]
            #[cfg_attr(docsrs, doc(
                cfg(feature = "pulse")
            ))]
            $item
        )*
    }
}

macro_rules! cfg_pipewire {
    ($($item:item)*) => {
        $(
            #[cfg(feature = "pipewire")]
            #[cfg_attr(docsrs, doc(
                cfg(feature = "pipewire")
            ))]
            $item
        )*
    }
}
