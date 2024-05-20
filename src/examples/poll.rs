use std::{io::stdin, os::fd::AsFd};

/// Section 7.2 "poll() - Synchonous I/O Multiplexing"
///
/// Bindings: `nix`
///
/// Ask the OS to do the dirty work of polling for us,
/// to avoid consuming too much CPU time.
/// When polling, the process is put to sleep until the FD is ready.
///
/// Original: [poll.c](https://beej.us/guide/bgnet/examples/poll.c)
pub fn pollstdin() {
    let stfd = stdin();
    let pfd = nix::poll::PollFd::new(stfd.as_fd(), nix::poll::PollFlags::POLLIN);
    let mut fds = [pfd];
    let num_events = nix::poll::poll(&mut fds, 2500 as u16).expect("poll failed");

    if num_events == 0 {
        println!("No events");
        return;
    } else {
        if let Some(e) = pfd.revents() {
            println!("FD ready to read:\n{:?}", pfd);
            println!("Event flags: {:?}", e);
        } else {
            println!("No events");
        }
    }
}
