use crate::loom::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let events = audio_device::runtime::Events::new()?;
    let event = Arc::new(events.event(false)?);
    let event2 = event.clone();

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        event2.set();
    });

    println!("waiting for event...");
    event.wait().await;
    println!("event woken up");

    events.join();
    Ok(())
}
