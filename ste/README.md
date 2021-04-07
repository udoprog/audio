# ste

[![Documentation](https://docs.rs/ste/badge.svg)](https://docs.rs/ste)
[![Crates](https://img.shields.io/crates/v/ste.svg)](https://crates.io/crates/ste)
[![Actions Status](https://github.com/udoprog/rotary/workflows/Rust/badge.svg)](https://github.com/udoprog/rotary/actions)

A single-threaded executor with some tricks up its sleeve.

This was primarily written for use in [rotary] as a low-latency way of
interacting with a single background thread for audio-related purposes, but
is otherwise a general purpose library that can be used by anyone.

> **Soundness Warning:** This crate uses a fair bit of **unsafe**. Some of
> the tricks employed needs to be rigirously sanity checked for safety
> before you can rely on this for production uses.

The default way to access the underlying thread is through the [submit]
method. This blocks the current thread for the duration of the task allowing
the background thread to access variables which are in scope. Like `n`
below.

```rust
let thread = ste::Thread::new()?;

let mut n = 10;
thread.submit(|| n += 10)?;
assert_eq!(20, n);

thread.join()?;
```

## Restricting which threads can access data

We provide the [Tagged] container. Things stored in this container may
*only* be accessed by the thread in which the container was created.

It works by associating a tag with the data that is unique to the thread
which created it. Any attempt to access the data will check this tag against
the tag in the current thread.

```rust
struct Foo;

impl Foo {
    fn say_hello(&self) {
        println!("Hello World!");
    }
}

let thread = ste::Thread::new()?;

let foo = thread.submit(|| ste::Tagged::new(Foo))?;
foo.say_hello(); // <- Panics!

thread.join()?;
```

Using it inside of the thread that created it is fine.

```rust
let thread = ste::Thread::new()?;

let foo = thread.submit(|| ste::Tagged::new(Foo))?;

thread.submit(|| {
    foo.say_hello(); // <- OK!
})?;

thread.join()?;
```

> There are some other details you need to know relevant to how to use the
> [Tagged] container. See its documentation for more.

## Known unsafety and soundness issues

Below you can find a list of known soundness issues this library currently
has.

### Pointers to stack-local addresses

In order to efficiently share data between a thread calling [submit] and the
background thread, the background thread references a fair bit of
stack-local data from the calling thread which involves a fair bit of
`unsafe`.

While it should be possible to make this use *safe* (as is the hope of this
library), it carries a risk that if the background thread were to proceed
executing a task that is no longer synchronized properly with a caller of
[submit] it might end up referencing data which is either no longer valid
(use after free), or contains something else (dirty).

### Soundness issue with tag re-use

[Tagged] containers currently use a tag based on the address of a slab of
allocated memory that is associated with each [Thread]. If however a
[Thread] is shut down, and a new later recreated, there is a slight risk
that this might re-use an existing memory address.

Memory addresses are quite thankful to use, because they're cheap and quite
easy to access. Due to this it might however be desirable to use a generated
ID per thread instead which can for example abort a program in case it can't
guarantee uniqueness.

[submit]: https://docs.rs/ste/0/ste/struct.Thread.html#method.submit
[Thread]: https://docs.rs/ste/0/ste/struct.Thread.html
[Tagged]: https://docs.rs/ste/0/ste/struct.Tagged.html
[rotary]: https://github.com/udoprog/rotary

License: MIT/Apache-2.0
