fn main() -> anyhow::Result<()> {
    for _ in 0..10 {
        let thread = ste::spawn();
        let mut result = 0;

        for n in 0..100 {
            result += thread.submit(move || n);
        }

        assert_eq!(result, 4950);
        thread.join();
    }

    Ok(())
}
