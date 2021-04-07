use anyhow::anyhow;
use std::sync::Arc;
use std::thread;

#[test]
fn test_recover_from_panic() -> anyhow::Result<()> {
    for _ in 0..100 {
        let thread = Arc::new(crate::Thread::new()?);

        let mut threads = Vec::new();

        for _ in 0..10 {
            let thread = thread.clone();

            threads.push(thread::spawn(move || {
                thread.submit(|| {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    panic!("trigger");
                })
            }));
        }

        for t in threads {
            let t = t.join().unwrap();
            assert!(t.is_err());
        }

        let thread = Arc::try_unwrap(thread).map_err(|_| anyhow!("unwrap failed"))?;
        assert!(thread.join().is_err());
    }

    Ok(())
}
