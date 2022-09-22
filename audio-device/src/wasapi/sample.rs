use std::mem;
use windows::Win32::Media::Audio as audio;
use windows::Win32::Media::KernelStreaming as ks;
use windows::Win32::Media::Multimedia as mm;

use super::ClientConfig;

/// Trait implemented for types which can be used to feed the output device.
pub unsafe trait Sample: Copy {
    /// The mid level (silent) level of a sample.
    const MID: Self;

    /// Construct a wave format specification compatible with the current sample
    /// type.
    fn mix_format(config: ClientConfig) -> audio::WAVEFORMATEXTENSIBLE;

    /// Test if the current sample type is compatible.
    unsafe fn is_compatible_with(mix_format: *const audio::WAVEFORMATEX) -> bool;
}

unsafe impl Sample for i16 {
    const MID: Self = 0;

    fn mix_format(config: ClientConfig) -> audio::WAVEFORMATEXTENSIBLE {
        let avg_bytes_per_sec =
            config.channels as u32 * config.sample_rate * mem::size_of::<Self>() as u32;
        let bits_per_sample = mem::size_of::<Self>() as u16 * 8;
        let block_align = config.channels as u16 * mem::size_of::<Self>() as u16;

        audio::WAVEFORMATEXTENSIBLE {
            Format: audio::WAVEFORMATEX {
                wFormatTag: audio::WAVE_FORMAT_PCM as u16,
                nChannels: config.channels,
                nSamplesPerSec: config.sample_rate,
                nAvgBytesPerSec: avg_bytes_per_sec,
                nBlockAlign: block_align,
                wBitsPerSample: bits_per_sample,
                cbSize: 0,
            },
            Samples: audio::WAVEFORMATEXTENSIBLE_0 {
                wValidBitsPerSample: 0,
            },
            dwChannelMask: 0,
            SubFormat: windows::core::GUID::zeroed(),
        }
    }

    unsafe fn is_compatible_with(mix_format: *const audio::WAVEFORMATEX) -> bool {
        let bits_per_sample = (*mix_format).wBitsPerSample;

        match (*mix_format).wFormatTag as u32 {
            audio::WAVE_FORMAT_PCM => {
                assert!((*mix_format).cbSize == 0);
                bits_per_sample == 16
            }
            _ => false,
        }
    }
}

unsafe impl Sample for f32 {
    const MID: Self = 0.0;

    fn mix_format(config: ClientConfig) -> audio::WAVEFORMATEXTENSIBLE {
        let avg_bytes_per_sec =
            config.channels as u32 * config.sample_rate * mem::size_of::<Self>() as u32;
        let bits_per_sample = mem::size_of::<Self>() as u16 * 8;
        let block_align = config.channels as u16 * mem::size_of::<Self>() as u16;
        let cb_size = mem::size_of::<audio::WAVEFORMATEXTENSIBLE>() as u16
            - mem::size_of::<audio::WAVEFORMATEX>() as u16;

        audio::WAVEFORMATEXTENSIBLE {
            Format: audio::WAVEFORMATEX {
                wFormatTag: ks::WAVE_FORMAT_EXTENSIBLE as u16,
                nChannels: config.channels,
                nSamplesPerSec: config.sample_rate,
                nAvgBytesPerSec: avg_bytes_per_sec,
                nBlockAlign: block_align,
                wBitsPerSample: bits_per_sample,
                cbSize: cb_size,
            },
            Samples: audio::WAVEFORMATEXTENSIBLE_0 {
                wValidBitsPerSample: bits_per_sample,
            },
            dwChannelMask: 0,
            SubFormat: mm::KSDATAFORMAT_SUBTYPE_IEEE_FLOAT,
        }
    }

    unsafe fn is_compatible_with(mix_format: *const audio::WAVEFORMATEX) -> bool {
        let bits_per_sample = (*mix_format).wBitsPerSample;

        match (*mix_format).wFormatTag as u32 {
            ks::WAVE_FORMAT_EXTENSIBLE => {
                debug_assert_eq! {
                    (*mix_format).cbSize as usize,
                    mem::size_of::<audio::WAVEFORMATEXTENSIBLE>() - mem::size_of::<audio::WAVEFORMATEX>()
                };

                let mix_format = mix_format as *const audio::WAVEFORMATEXTENSIBLE;
                bits_per_sample == 32
                    && matches!((*mix_format).SubFormat, mm::KSDATAFORMAT_SUBTYPE_IEEE_FLOAT)
            }
            _ => false,
        }
    }
}
