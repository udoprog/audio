use std::sync::Arc;
use std::thread;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    for _ in 0..10 {
        let mut threads = Vec::new();

        let thread = Arc::new(ste::Thread::new()?);

        for n in 0..100 {
            let thread = thread.clone();

            threads.push(thread::spawn(move || {
                let mut result = 0u32;

                let future = thread.submit_async(async {
                    result += n;
                });

                futures::executor::block_on(future).unwrap();
                result
            }));
        }

        let mut result = 0;

        for t in threads {
            result += t.join().unwrap();
        }

        assert_eq!(result, 4950);

        let thread = Arc::try_unwrap(thread).map_err(|_| "not unique").unwrap();
        thread.join()?;
    }

    Ok(())
}
