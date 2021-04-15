#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    for _ in 0..10u32 {
        let thread = ste::spawn();
        let mut result = 0u32;

        for n in 0..100 {
            thread
                .submit_async(async {
                    result += n;
                })
                .await;
        }

        assert_eq!(result, 4950);
        thread.join();
    }

    let thread = ste::spawn();
    thread.submit_async(async move { panic!("woops") }).await;
    Ok(())
}
