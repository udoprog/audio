use std::sync::Arc;
use std::thread;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    for _ in 0..10 {
        let mut threads = Vec::new();

        let audio_thread = Arc::new(ste::Thread::new()?);

        for n in 0..100 {
            let audio_thread = audio_thread.clone();

            threads.push(thread::spawn(move || {
                let mut result = 0u32;

                let future = audio_thread.submit_async(async {
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

        let audio_thread = Arc::try_unwrap(audio_thread)
            .map_err(|_| "not unique")
            .unwrap();
        assert!(audio_thread.join().is_ok());
    }

    Ok(())
}
