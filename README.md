# rotary

A library for dealing efficiently with AudioBuffer non-interleaved audio
buffers.

```rust
use rand::Rng as _;

let mut buffer = rotary::AudioBuffer::<f32>::new();

buffer.resize_channels(2);
buffer.resize(2048);

/// Fill both channels with random noise.
let mut rng = rand::thread_rng();
rng.fill(&mut buffer[0]);
rng.fill(&mut buffer[1]);
```

## Creating and using a masked audio buffer.

```rust
use rotary::BitSet;

let mut buffer = rotary::MaskedAudioBuffer::<f32, BitSet<u128>>::with_topology(4, 1024);

buffer.mask(1);

for  channel in buffer.iter_mut() {
    for b in channel {
        *b = 1.0;
    }
}

let expected = vec![1.0f32; 1024];

assert_eq!(&buffer[0], &expected[..]);
assert_eq!(&buffer[1], &[][..]);
assert_eq!(&buffer[2], &expected[..]);
assert_eq!(&buffer[3], &expected[..]);
```

License: MIT/Apache-2.0
