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

            let t = thread::spawn(move || {
                thread.submit(|| {
                    thread::sleep(std::time::Duration::from_millis(10));
                    panic!("trigger");
                })
            });

            threads.push(t);
        }

        for t in threads {
            let t = t.join().unwrap();
            assert!(t.is_err());
        }

        let thread = Arc::try_unwrap(thread).map_err(|_| anyhow!("unwrap failed"))?;
        assert!(thread.join().is_ok());
    }

    Ok(())
}

#[test]
fn test_threading() -> anyhow::Result<()> {
    for _ in 0..100 {
        let thread = Arc::new(crate::Thread::new()?);

        let mut threads = Vec::new();

        for n in 0..10 {
            let thread = thread.clone();
            let t = thread::spawn(move || {
                thread.submit(|| {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                    n
                })
            });
            threads.push(t);
        }

        let mut result = 0;

        for t in threads {
            result += t.join().unwrap().unwrap();
        }

        assert_eq!(result, 45);

        let thread = Arc::try_unwrap(thread).map_err(|_| anyhow!("unwrap failed"))?;
        thread.join().unwrap();
    }

    Ok(())
}
