use std::{
    io::stdin,
    os::fd::{AsFd, AsRawFd},
};

/// Section 7.3 "select()—Synchronous I/O Multiplexing, Old School"
///
/// Bindings: `nix`
///
/// Wait for something to appear on standard input.
/// When running you need to write something and hit ENTER.
///
/// Original: [select.c]((https://beej.us/guide/bgnet/examples/select.c))
pub fn select() {
    // Create file descriptor 0 for stdin
    let stfd = stdin();
    let stfd = stfd.as_fd();

    // Create the FdSet for reading
    let mut readset = nix::sys::select::FdSet::new();
    readset.insert(stfd);
    // Timeout with seconds, microseconds
    let mut timeout = nix::sys::time::TimeVal::new(2, 500000);

    // Now for the select, for some reason we pass `nfds` as the highest file descriptor number plus 1,
    // from all the FdSets (in this case, our file descriptor is 0, so 0+1 =1)
    let selectout = nix::sys::select::select(
        stfd.as_raw_fd() + 1,
        Some(&mut readset),
        None,
        None,
        &mut timeout,
    )
    .expect("select failed");
    println!("selectout: {}", selectout);

    // Insetad of using FD_ISSET, we can just check if it contains our fd 0
    if readset.contains(stfd) {
        println!("Message came")
    } else {
        println!("Timed out");
    }
}
