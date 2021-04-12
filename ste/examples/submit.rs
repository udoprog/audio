fn main() -> anyhow::Result<()> {
    for _ in 0..10 {
        let thread = ste::Thread::new()?;
        let mut result = 0;

        for n in 0..100 {
            result += thread.submit(move || n)?;
        }

        assert_eq!(result, 4950);
        assert!(thread.join().is_ok());
    }

    Ok(())
}
