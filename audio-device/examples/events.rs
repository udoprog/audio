use audio_device::windows::AsyncEvent;
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let runtime = audio_device::runtime::Runtime::new()?;
    let guard = runtime.enter();
    let event = Arc::new(AsyncEvent::new(false)?);
    let event2 = event.clone();

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        event2.set();
    });

    println!("waiting for event...");
    event.wait().await;
    println!("event woken up");

    drop(guard);
    runtime.join();
    Ok(())
}
