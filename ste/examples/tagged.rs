use anyhow::Result;
use ste::{Tagged, Thread};

struct Foo(*mut ());

impl Foo {
    fn test(&self) -> u32 {
        42
    }
}

impl Drop for Foo {
    fn drop(&mut self) {
        println!("foo was dropped");
    }
}

fn main() -> Result<()> {
    let thread = Thread::new()?;

    let value = thread.submit(|| Tagged::new(Foo(0 as *mut ())))?;
    let out = thread.submit(|| value.test())?;
    assert_eq!(42, out);

    thread.drop(value)?;
    thread.join()?;
    Ok(())
}
