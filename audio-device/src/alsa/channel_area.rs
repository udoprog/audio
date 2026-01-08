// component is work in progress
#![allow(unused)]

use core::ffi::c_ulong;
use core::ptr;
use core::ptr::NonNull;

use alsa_sys as alsa;

/// A memory-mapped channel area.
pub struct ChannelArea<'a> {
    pub(super) pcm: &'a mut NonNull<alsa::snd_pcm_t>,
    pub(super) area: *const alsa::snd_pcm_channel_area_t,
    pub(super) offset: c_ulong,
    pub(super) frames: c_ulong,
}
