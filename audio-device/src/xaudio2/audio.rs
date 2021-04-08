use crate::bindings::Windows::Win32::XAudio2 as x2;

pub struct Audio {
    pub(super) audio: x2::IXAudio2,
}
