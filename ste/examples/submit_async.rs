#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    for _ in 0..10 {
        let audio_thread = ste::Thread::new()?;
        let mut result = 0u32;

        for n in 0..100 {
            audio_thread
                .submit_async(async {
                    result += n;
                })
                .await?;
        }

        assert_eq!(result, 4950);
        assert!(audio_thread.join().is_ok());
    }

    let audio_thread = ste::Thread::new()?;

    let result = audio_thread
        .submit_async(async move { panic!("woops") })
        .await;

    assert!(result.is_err());
    assert!(audio_thread.join().is_err());
    Ok(())
}
