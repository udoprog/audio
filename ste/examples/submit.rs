fn main() -> anyhow::Result<()> {
    for _ in 0..1000 {
        let audio_thread = ste::Thread::new()?;
        let mut result = 0;

        for n in 0..100 {
            result += audio_thread.submit(move || n)?;
        }

        assert_eq!(result, 4950);
        assert!(audio_thread.join().is_ok());
    }

    Ok(())
}
