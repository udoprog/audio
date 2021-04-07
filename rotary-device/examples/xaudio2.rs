#[cfg(not(all(windows, feature = "xaudio2")))]
pub fn main() {
    println!("xaudio2 support is not enabled");
}

#[cfg(all(windows, feature = "xaudio2"))]
pub fn main() -> Result<()> {
    let audio_thread = ste::Builder::new().prelude(wasapi::audio_prelude).build()?;

    audio_thread.submit(|| {
        let output = rotary_device::xaudio2::default_audio()?;
        let output = output.ok_or_else(|| anyhow!("no default output device found"))?;

        Ok::<(), anyhow::Error>(())
    })??;

    audio_thread.join()?;
    Ok(())
}
