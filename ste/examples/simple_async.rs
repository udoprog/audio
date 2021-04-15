#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let thread = ste::Builder::new().with_tokio().build()?;

    let mut result = 0u32;

    thread
        .submit_async(async {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            result += 1
        })
        .await;

    assert_eq!(result, 1u32);

    thread.join();
    Ok(())
}
