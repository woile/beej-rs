use std::{
    net::{Ipv6Addr, SocketAddrV6},
    os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd},
};

/// Section 7.3 "select()—Synchronous I/O Multiplexing, Old School"
///
/// Simple multi-user chat server
///
/// Bindings: `nix`
///
/// Protocol: `TCP`
///
/// Usage
///
/// ```
/// telnet localhost 9034
/// ```
///
/// Use CTRL+5 and then type "quit" to close the connection
/// On Mac CTRL + C doesn't work
///
/// Original: [selectserver.c](https://beej.us/guide/bgnet/examples/select.c)
pub fn select_server(port: u16) {
    // get us a socket in 0.0.0.0
    let unspec = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, port, 0, 0);
    let socket: nix::sys::socket::SockaddrIn6 = unspec.into();
    let listener = nix::sys::socket::socket(
        nix::sys::socket::AddressFamily::Inet6,
        nix::sys::socket::SockType::Stream,
        nix::sys::socket::SockFlag::empty(),
        None,
    )
    .expect("Failed to create socket");

    // Lose the "address already in use" error message
    nix::sys::socket::setsockopt(&listener, nix::sys::socket::sockopt::ReuseAddr, &true)
        .expect("Failed to set socket options");

    nix::sys::socket::bind(listener.as_raw_fd(), &socket).expect("Failed to bind to socket");
    let backlog = nix::sys::socket::Backlog::new(10).expect("Failed to create backlog");
    nix::sys::socket::listen(&listener, backlog).expect("Failed to listen on socket");

    let mut main = nix::sys::select::FdSet::new();
    main.insert(listener.as_fd());

    println!("Listening on {}", unspec);

    loop {
        // copy the main set, so we don't lose the listener when doing select
        // remember that select cleans the fd from the set which had no events
        // let mut read_fds = nix::sys::select::FdSet::clone(&main);
        // read_fds.clone_from(&main);
        let mut read_fds = main.clone();

        // I'm not happy with the fdmax + 1, when we can just pass None and let
        // select calculate it for us
        // we also set timeout to None to block indefinetly
        // we also don't care about the number of events in the output
        let _ = nix::sys::select::select(None, Some(&mut read_fds), None, None, None)
            .expect("Failed to select...");

        let active_fd: Vec<RawFd> = read_fds
            .fds(None)
            .map(|borrowed_fd| borrowed_fd.as_raw_fd())
            .collect();
        active_fd.iter().for_each(|fd| {
            if fd.as_raw_fd() == listener.as_raw_fd() {
                // Handle new connections against the listener
                println!("[Server] Starting new connection...");
                let new_rfd = nix::sys::socket::accept(listener.as_raw_fd())
                    .expect("Failed to accept new conn");
                let new_fd = unsafe { BorrowedFd::borrow_raw(new_rfd) };
                let ss: nix::sys::socket::SockaddrStorage =
                    nix::sys::socket::getpeername(new_fd.as_raw_fd()).expect("getpeername failed");
                main.insert(new_fd);
                println!(
                    "[Server] New connection {}",
                    ss.as_sockaddr_in6().expect("sockaddr not ipv6")
                );
            } else {
                let mut buf = [0u8; 1024];
                // Handle existing connection
                let nbytes = nix::sys::socket::recv(
                    fd.as_raw_fd(),
                    &mut buf,
                    nix::sys::socket::MsgFlags::empty(),
                )
                .expect("failed to receive data");

                // nbyes is unsigned, in C nbytes might be negative,
                // which means ther was an error, but in rust,
                // recv returns a Result, and we have already handled the errors with expect.
                // Maybe it's not the best idea to use expect in this case, as it will shut down
                // the application, but YOLO
                match nbytes {
                    0 => {
                        println!("connection closed by socket!");
                        let new_fd = unsafe { BorrowedFd::borrow_raw(fd.as_raw_fd()) };
                        main.remove(new_fd);
                    }
                    _ => {
                        main.fds(None).for_each(|mfd| {
                            if mfd.as_raw_fd() != listener.as_raw_fd()
                                && mfd.as_raw_fd() != fd.as_raw_fd()
                            {
                                let r = nix::sys::socket::send(
                                    mfd.as_raw_fd(),
                                    &mut buf,
                                    nix::sys::socket::MsgFlags::empty(),
                                );
                                if let Err(e) = r {
                                    eprint!("{}", e);
                                }
                            }
                        });
                    }
                }
            }
        });
    }
}
