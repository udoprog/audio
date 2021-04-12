#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    for _ in 0..10 {
        let thread = ste::Thread::new()?;
        let mut result = 0u32;

        for n in 0..100 {
            thread
                .submit_async(async {
                    result += n;
                })
                .await?;
        }

        assert_eq!(result, 4950);
        assert!(thread.join().is_ok());
    }

    let thread = ste::Thread::new()?;

    let result = thread.submit_async(async move { panic!("woops") }).await;

    assert!(result.is_err());
    assert!(thread.join().is_err());
    Ok(())
}
