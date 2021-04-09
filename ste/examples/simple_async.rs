#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let audio_thread = ste::Thread::new()?;
    let result = 42u32;
    audio_thread.submit_async(async move { result }).await?;
    assert_eq!(result, 42u32);
    assert!(audio_thread.join().is_ok());

    Ok(())
}
