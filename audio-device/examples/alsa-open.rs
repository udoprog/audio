use audio_device::alsa;

fn main() -> anyhow::Result<()> {
    let pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;

    let hw = pcm.hardware_parameters_any()?;

    println!("Rate Resample: {}", hw.rate_resample(&pcm)?);
    println!("Channels: {}", hw.channels()?);
    println!("Test Channels (2): {}", hw.test_channels(&pcm, 2)?);
    println!("Min Rate: {}", hw.rate_min()?);
    println!("Max Rate: {}", hw.rate_max()?);
    println!("Rate: {}", hw.rate()?);
    println!("Format: {}", hw.format()?);
    println!("Access: {}", hw.access()?);

    Ok(())
}
