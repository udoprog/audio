//! An idiomatic Rust ALSA interface.
// Documentation: https://www.alsa-project.org/alsa-doc/alsa-lib/

mod error;
pub use self::error::{Error, Result};

mod c_string;
pub use self::c_string::CString;

mod card;
pub use self::card::{Card, cards};

mod pcm;
pub use self::pcm::Pcm;

mod hardware_parameters;
pub use self::hardware_parameters::{HardwareParameters, HardwareParametersMut};

mod software_parameters;
pub use self::software_parameters::{SoftwareParameters, SoftwareParametersMut};

mod format_mask;
pub use self::format_mask::FormatMask;

mod access_mask;
pub use self::access_mask::AccessMask;

mod enums;
pub use self::enums::{
    Access, ControlElementInterface, Direction, Format, State, Stream, Timestamp, TimestampType,
};

mod channel_area;
#[doc(hidden)]
pub use self::channel_area::ChannelArea;

mod writer;
pub use self::writer::Writer;

#[cfg(feature = "poll-driver")]
mod async_writer;
#[cfg(feature = "poll-driver")]
#[cfg_attr(docsrs, doc(cfg(feature = "poll-driver")))]
pub use self::async_writer::AsyncWriter;

mod sample;
pub use self::sample::Sample;

mod configurator;
pub use self::configurator::{Config, Configurator};

mod control;
pub use self::control::{Control, ControlElementList};
