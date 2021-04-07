use anyhow::Result;
use rotary_device::{AudioThread, Panicked};
use std::sync::Arc;

fn main() -> Result<()> {
    let audio_thread = Arc::new(AudioThread::new()?);
    let mut threads = Vec::new();

    for n in 0..10 {
        let audio_thread = audio_thread.clone();

        threads.push(std::thread::spawn(move || {
            let mut v = 0;
            audio_thread.submit(|| v += n)?;
            Ok::<_, Panicked>(v)
        }));
    }

    let mut result = 0;

    for t in threads {
        result += t.join().unwrap()?;
    }

    assert_eq!(result, (0..10).sum());

    // Unwrap the audio thread.
    let audio_thread = Arc::try_unwrap(audio_thread)
        .map_err(|_| "unwrap failed")
        .unwrap();

    let value = audio_thread.submit(|| {
        panic!("Audio thread: {:?}", std::thread::current().id());
    });

    println!("Main thread: {:?}", std::thread::current().id());
    assert!(value.is_err());
    assert!(audio_thread.join().is_err());
    Ok(())
}
