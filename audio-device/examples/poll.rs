#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let poll = audio_device::driver::Poll::new()?;
    poll.test()?;

    std::thread::sleep(std::time::Duration::from_secs(10));

    Ok(())
}
