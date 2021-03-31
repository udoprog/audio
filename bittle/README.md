# bittle

A library for working with small and cheap bit sets and masks

Masks keep track of usize indexes which are set through
[testing][Mask::test]. This allows for masking indexes in certain
operations. Like if you want to mask which channels in an audio buffer is in
use or not.

License: MIT/Apache-2.0
