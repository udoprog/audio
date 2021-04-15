#![feature(test)]

extern crate test;

use test::Bencher;

#[bench]
fn test_on_ste(b: &mut Bencher) {
    b.iter(|| {
        let thread = ste::spawn().unwrap();
        let mut result = 0;

        for n in 0..100 {
            result += thread.submit(move || n).unwrap();
        }

        assert!(thread.join().is_ok());
        result
    });
}

#[bench]
fn count_to_1000_ste(b: &mut Bencher) {
    b.iter(|| {
        let thread = ste::spawn().unwrap();
        let mut total = 0u32;

        for n in 0..1000u32 {
            total += thread.submit(move || n + 1).unwrap();
        }

        thread.join().unwrap();
        assert_eq!(total, 500500);
        total
    });
}

#[bench]
fn count_to_1000_mpsc(b: &mut Bencher) {
    use std::sync::mpsc;
    use std::thread;

    b.iter(|| {
        let mut total = 0u32;

        let t = {
            let (tx, rx) = mpsc::sync_channel(0);
            let (out_tx, out_rx) = mpsc::sync_channel(0);

            let t = thread::spawn(move || {
                while let Ok(task) = rx.recv() {
                    out_tx.send(task + 1).unwrap();
                }
            });

            for n in 0..1000u32 {
                tx.send(n).unwrap();
                total += out_rx.recv().unwrap();
            }

            t
        };

        t.join().unwrap();
        assert_eq!(total, 500500);
        total
    });
}
