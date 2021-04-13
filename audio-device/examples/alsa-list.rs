fn main() -> anyhow::Result<()> {
    for card in audio_device::alsa::cards() {
        let card = card?;
        println!(
            "{} ({})",
            card.long_name()?.to_str()?,
            card.name()?.to_str()?
        );
    }

    Ok(())
}
