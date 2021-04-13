use audio_device::alsa;

fn main() -> anyhow::Result<()> {
    let mut pcm = alsa::Pcm::open_default(alsa::Stream::Playback)?;

    let mut hw = pcm.hardware_parameters_any()?;
    hw.set_channels_near(1)?;
    let rate = hw.set_rate_minmax(
        48000,
        alsa::Direction::Nearest,
        48000,
        alsa::Direction::Nearest,
    )?;
    dbg!(rate);

    let format = hw.set_format_first()?;
    dbg!(format);

    hw.install()?;

    let hw = pcm.hardware_parameters_current()?;

    println!("Channels: {}", hw.channels()?);
    println!("Rate: {}", hw.rate()?);
    println!("Min Rate: {}", hw.rate_min()?);
    println!("Max Rate: {}", hw.rate_max()?);
    println!("Rate: {}", hw.rate()?);
    println!("Format: {}", hw.format()?);
    println!("Access: {}", hw.access()?);

    Ok(())
}
