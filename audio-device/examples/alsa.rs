use audio_core::ReadBuf;
use audio_core::Translate as _;
use audio_device::alsa;
use audio_generator::{self as gen, Generator as _};

fn main() -> anyhow::Result<()> {
    let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;

    let config = pcm.configure::<i16>().install()?;
    let mut writer = pcm.writer::<i16>()?;
    dbg!(config);

    let sample_rate = config.rate as f32;
    let channels = config.channels as usize;

    let mut a = gen::Sin::new(261.63, sample_rate);
    let mut b = gen::Sin::new(329.63, sample_rate);
    let mut c = gen::Sin::new(440.00, sample_rate);
    let mut buf = [0i16; 16 * 1024];

    loop {
        for o in (0..buf.len()).step_by(2) {
            let s = i16::translate((a.sample() + b.sample() + c.sample()) * 0.01);

            for c in 0..channels {
                buf[o + c] = s;
            }
        }

        let mut buf = audio::wrap::interleaved(&buf[..], channels);

        while buf.has_remaining() {
            writer.write_interleaved(&mut buf)?;
        }
    }
}
