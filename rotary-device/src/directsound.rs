use bindings::Windows::Win32::Audio::{
    DirectSoundCreate, IDirectSound, IDirectSoundBuffer, DSBUFFERDESC, DSCAPS, DSCAPS_CERTIFIED,
    DSCAPS_CONTINUOUSRATE, DSCAPS_PRIMARY16BIT, DSCAPS_PRIMARY8BIT, DSSCL_NORMAL,
};
use bindings::Windows::Win32::Multimedia::WAVEFORMATEX;
use bindings::Windows::Win32::WindowsAndMessaging::GetForegroundWindow;
use std::mem;
use std::ptr;
use thiserror::Error;
use windows::Interface;

/// WASAPI-specific errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Windows error")]
    Io(#[from] windows::Error),
}

/// A windows host.
#[derive(Clone)]
pub struct Device {
    device: IDirectSound,
    buffer: IDirectSoundBuffer,
}

impl Device {
    /// Construct a raw device.
    fn from_raw(device: IDirectSound, buffer: IDirectSoundBuffer) -> Self {
        Self { device, buffer }
    }
}

/// Open the default input device for WASAPI.
pub fn default_output_device() -> Result<Option<Device>, Error> {
    unsafe {
        let mut device = None;
        DirectSoundCreate(std::ptr::null_mut(), &mut device, None).ok()?;

        let device = match device {
            Some(device) => device,
            None => return Ok(None),
        };

        let hwnd = GetForegroundWindow();

        device.SetCooperativeLevel(hwnd, DSSCL_NORMAL).ok()?;

        let mut desc = DSBUFFERDESC {
            dwSize: std::mem::size_of::<DSBUFFERDESC>() as u32,
            dwFlags: 1,
            ..Default::default()
        };

        let mut caps = DSCAPS {
            dwSize: std::mem::size_of::<DSCAPS>() as u32,
            ..DSCAPS::default()
        };

        device.GetCaps(&mut caps).ok()?;

        println!("caps: {:0b}", caps.dwFlags);

        if caps.dwFlags & DSCAPS_CONTINUOUSRATE != 0 {
            println!(
                "from: {}, to: {}",
                caps.dwMinSecondarySampleRate, caps.dwMaxSecondarySampleRate
            );
        }

        if caps.dwFlags & DSCAPS_PRIMARY16BIT != 0 {
            println!("16 bit audio!");
        }

        if caps.dwFlags & DSCAPS_PRIMARY8BIT != 0 {
            println!("8 bit audio!");
        }

        let mut buffer = None;

        let buffer = device
            .CreateSoundBuffer(&mut desc, &mut buffer, None)
            .and_some(buffer)?;

        let mut format = mem::MaybeUninit::zeroed();
        let mut written = mem::MaybeUninit::zeroed();

        buffer
            .GetFormat(
                format.as_mut_ptr(),
                mem::size_of::<WAVEFORMATEX>() as u32,
                written.as_mut_ptr(),
            )
            .ok()?;

        let format = format.assume_init();
        let written = written.assume_init();

        dbg!(format.nChannels);
        dbg!(format.nSamplesPerSec);
        dbg!(format.wBitsPerSample);
        dbg!(format.wFormatTag);

        Ok(Some(Device::from_raw(device, buffer)))
    }
}
