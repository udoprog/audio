#[cfg(windows)]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use std::sync::Arc;

    let handle = audio_device::driver::events::Handle::new()?;
    let event = Arc::new(handle.event(false)?);
    let event2 = event.clone();

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        event2.set();
    });

    println!("waiting for event...");
    event.wait().await;
    println!("event woken up");

    handle.join()?;
    Ok(())
}

#[cfg(not(windows))]
fn main() {
    println!("events example is only supported on windows");
}
