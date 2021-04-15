use audio_device::driver::{Poll, PollHandle};
use std::fs::OpenOptions;
use std::io;
use std::io::Read;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;

/// Read the data from the reader `R` asynchronously using the specified poll
/// handle.
async fn read_to_vec<R>(handle: &PollHandle, mut read: R) -> io::Result<Vec<u8>>
where
    R: Read,
{
    let mut data = Vec::<u8>::new();
    let mut buf = [0u8; 1024];

    'outer: loop {
        let guard = handle.returned_events().await;
        let events = guard.events();

        if events & libc::POLLIN != 0 {
            loop {
                match read.read(&mut buf[..]) {
                    Ok(0) => break 'outer,
                    Ok(n) => {
                        data.extend(&buf[..n]);
                    }
                    // Note: This will most likely never happen, because
                    // nonblocking I/O on Linux file descriptors are never
                    // signalled as blocking.
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        continue 'outer;
                    }
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }
        } else {
            panic!("did not grok the correct error kind here: {:?}", events);
        }
    }

    Ok(data)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let poll = Poll::new()?;

    let mut file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NONBLOCK)
        .open("Cargo.toml")?;

    let pollfd = libc::pollfd {
        fd: file.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };

    let handle = poll.register(pollfd)?;
    let data = read_to_vec(&handle, &mut file).await?;

    poll.join();

    let contents = std::str::from_utf8(&data)?;
    println!("contents:\n{}", contents);
    Ok(())
}
