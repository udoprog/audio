#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    for _ in 0..10usize {
        let audio_thread = ste::Thread::new()?;
        let mut result = 0u32;

        for n in 0..100u32 {
            result += audio_thread.submit_async(async move { n }).await?;
        }

        assert_eq!(result, 4950);
        assert!(audio_thread.join().is_ok());
    }

    Ok(())
}
