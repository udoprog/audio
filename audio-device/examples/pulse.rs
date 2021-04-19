use audio_device::pulse;
use std::ffi::CString;

fn generate_audio() -> anyhow::Result<()> {
    let name = CString::new("Hello World")?;

    let mut main = pulse::MainLoop::new();
    let mut context = main.context(&name);

    context.set_callback(|c| {
        println!("state changed: {}", c.state()?);
        Err(pulse::Error::User("hello".into()))
    })?;

    context.connect()?;
    std::thread::sleep(std::time::Duration::from_secs(10));
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let bg = ste::spawn();
    bg.submit(generate_audio)?;
    bg.join();
    Ok(())
}
