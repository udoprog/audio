# ste

[![Documentation](https://docs.rs/ste/badge.svg)](https://docs.rs/ste)
[![Crates](https://img.shields.io/crates/v/ste.svg)](https://crates.io/crates/ste)
[![Actions Status](https://github.com/udoprog/audio/workflows/Rust/badge.svg)](https://github.com/udoprog/audio/actions)

A single-threaded executor with some tricks up its sleeve.

This was primarily written for use in [audio] as a low-latency way of
interacting with a single background thread for audio-related purposes, but
is otherwise a general purpose library that can be used to do anything.

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

## Restricting thread access using tags

This library provides the ability to construct a [Tag] which is uniquely
associated with the thread that created it. This can then be used to ensure
that data is only accessed on any one given thread.

This is useful, because many APIs requires *thread-locality*. Instances can
only safely be used by the thread that created them. This is a low-level
tool we provide which allows the safe implementation of `Send` for types
which are otherwise `!Send`.

Note that correctly using a [Tag] is hard, and incorrect use has sever
safety implications. Make sure to study its documentation closely before
use.

```rust
struct Foo {
    tag: ste::Tag,
}

impl Foo {
    fn new() -> Self {
        Self {
            tag: ste::Tag::current_thread(),
        }
    }

    fn say_hello(&self) {
        self.tag.ensure_on_thread();
        println!("Hello World!");
    }
}

let thread = ste::Thread::new()?;

let foo = thread.submit(|| Foo::new())?;
foo.say_hello(); // <- Panics!

thread.join()?;
```

Using `say_hello` inside of the thread that created it is however fine.

```rust
let thread = ste::Thread::new()?;

let foo = thread.submit(|| Foo::new())?;

thread.submit(|| {
    foo.say_hello(); // <- OK!
})?;

thread.join()?;
```

## Known unsafety and soundness issues

Below you can find a list of unsafe use and known soundness issues this
library currently has. The soundness issues **must be fixed** before this
library goes out of *alpha*.

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

### Tag re-use

[Tag] containers currently use a tag based on the address of a slab of
allocated memory that is associated with each [Thread]. If however a
[Thread] is shut down, and a new later recreated, there is a slight risk
that this might re-use an existing memory address.

Memory addresses are quite thankful to use, because they're cheap and quite
easy to access. Due to this it might however be desirable to use a generated
ID per thread instead which can for example abort a program in case it can't
guarantee uniqueness.

[submit]: https://docs.rs/ste/0.1.0-alpha.7/ste/struct.Thread.html#method.submit
[Thread]: https://docs.rs/ste/0.1.0-alpha.7/ste/struct.Thread.html
[Tag]: https://docs.rs/ste/0.1.0-alpha.7/ste/struct.Tag.html
[audio]: https://github.com/udoprog/audio

License: MIT/Apache-2.0
