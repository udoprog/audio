#![feature(test)]

extern crate test;

use test::Bencher;

#[bench]
fn test_on_ste(b: &mut Bencher) {
    b.iter(|| {
        let audio_thread = ste::Thread::new().unwrap();
        let mut result = 0;

        for n in 0..100 {
            result += audio_thread.submit(move || n).unwrap();
        }

        assert!(audio_thread.join().is_ok());
        result
    });
}
