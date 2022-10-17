# ste

[<img alt="github" src="https://img.shields.io/badge/github-udoprog/audio-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/audio)
[<img alt="crates.io" src="https://img.shields.io/crates/v/ste.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/ste)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-ste-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/ste)
[<img alt="build status" src="https://img.shields.io/github/workflow/status/udoprog/audio/CI/main?style=for-the-badge" height="20">](https://github.com/udoprog/audio/actions?query=branch%3Amain)

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
let thread = ste::spawn();

let mut n = 10;
thread.submit(|| n += 10);
assert_eq!(20, n);

thread.join();
```

<br>

## Restricting thread access using tags

This library provides the ability to construct a [Tag] which is uniquely
associated with the thread that created it. This can then be used to ensure
that data is only accessible by one thread.

This is useful, because many APIs requires *thread-locality* where instances
can only safely be used by the thread that created them. This is a low-level
tool we provide which allows the safe implementation of `Send` for types
which are otherwise `!Send`.

Note that correctly using a [Tag] is hard, and incorrect use has severe
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

let thread = ste::spawn();

let foo = thread.submit(|| Foo::new());

thread.submit(|| {
    foo.say_hello(); // <- OK!
});

thread.join();
```

Using `say_hello` outside of the thread that created it is not fine and will
panic to prevent racy access:

```rust
let thread = ste::spawn();

let foo = thread.submit(|| Foo::new());

foo.say_hello(); // <- Oops, panics!

thread.join();
```

<br>

## Known unsafety and soundness issues

Below you can find a list of unsafe use and known soundness issues this
library currently has. The soundness issues **must be fixed** before this
library goes out of *alpha*.

<br>

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

<br>

### Tag re-use

[Tag] containers currently use a tag based on the address of a slab of
allocated memory that is associated with each [Thread]. If however a
[Thread] is shut down, and a new later recreated, there is a slight risk
that this might re-use an existing memory address.

Memory addresses are quite thankful to use, because they're cheap and quite
easy to access. Due to this it might however be desirable to use a generated
ID per thread instead which can for example abort a program in case it can't
guarantee uniqueness.

[audio]: https://github.com/udoprog/audio
[submit]: https://docs.rs/ste/*/ste/struct.Thread.html#method.submit
[Tag]: https://docs.rs/ste/*/ste/struct.Tag.html
[Thread]: https://docs.rs/ste/*/ste/struct.Thread.html
