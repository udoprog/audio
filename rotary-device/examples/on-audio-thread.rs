use anyhow::Result;

fn main() -> Result<()> {
    let audio_thread = rotary_device::AudioThread::new()?;

    println!("main thread: {:?}", std::thread::current().id());

    let value = audio_thread.submit(|| {
        panic!("audio thread: {:?}", std::thread::current().id());
    });

    println!("main thread: {:?}", std::thread::current().id());
    println!("value: {:?}", value);

    audio_thread.join()?;
    Ok(())
}
