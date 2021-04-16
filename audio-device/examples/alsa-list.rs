use audio_device::alsa;
use std::ffi::CString;

fn main() -> anyhow::Result<()> {
    let thread = ste::spawn();

    thread.submit(|| {
        for card in alsa::cards() {
            let card = card?;
            let name = CString::new(format!("hw:{}", card.index()))?;
            let control = alsa::Control::open(&name)?;

            println!(
                "{} ({})",
                card.long_name()?.to_str()?,
                card.name()?.to_str()?
            );

            println!("control: {}", control.name().to_str()?);

            for (n, element) in control.element_list()?.iter().enumerate() {
                println!(
                    "{}: {} ({}) (index: {})",
                    n,
                    element.interface(),
                    element.name().to_str()?,
                    element.index(),
                );
            }
        }

        Ok::<_, anyhow::Error>(())
    })?;

    Ok(())
}
