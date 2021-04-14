// component is work in progress
#![allow(unused)]

use crate::libc as c;
use alsa_sys as alsa;
use std::ptr;

/// A memory-mapped channel area.
pub struct ChannelArea<'a> {
    pub(super) pcm: &'a mut ptr::NonNull<alsa::snd_pcm_t>,
    pub(super) area: *const alsa::snd_pcm_channel_area_t,
    pub(super) offset: c::c_ulong,
    pub(super) frames: c::c_ulong,
}
