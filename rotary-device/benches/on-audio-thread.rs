#![feature(test)]

extern crate test;

use test::Bencher;

#[bench]
fn test_single_thread(b: &mut Bencher) {
    b.iter(|| {
        let audio_thread = rotary_device::AudioThread::new().unwrap();
        let mut result = 0;

        for n in 0..100 {
            result += audio_thread.submit(move || n).unwrap();
        }

        assert!(audio_thread.join().is_ok());
        result
    });
}
