use criterion::{criterion_group, criterion_main, Criterion};

fn test_on_ste(b: &mut Criterion) {
    b.bench_function("test_on_ste", |b| {
        b.iter(|| {
            let thread = ste::spawn();
            let mut result = 0;

            for n in 0..100 {
                result += thread.submit(move || n);
            }

            thread.join();
            result
        });
    });
}

fn count_to_1000_ste(b: &mut Criterion) {
    b.bench_function("test_on_ste", |b| {
        b.iter(|| {
            let thread = ste::spawn();
            let mut total = 0u32;

            for n in 0..1000u32 {
                total += thread.submit(move || n + 1);
            }

            thread.join();
            assert_eq!(total, 500500);
            total
        });
    });
}

fn count_to_1000_mpsc(b: &mut Criterion) {
    use std::sync::mpsc;
    use std::thread;

    b.bench_function("count_to_1000_mpsc", |b| {
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
    });
}

criterion_group!(benches, test_on_ste, count_to_1000_ste, count_to_1000_mpsc);
criterion_main!(benches);
