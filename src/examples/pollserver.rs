use std::{
    net::{Ipv6Addr, SocketAddrV6},
    os::fd::{AsFd, AsRawFd, BorrowedFd},
};

/// Section 7.2 "poll() - Synchonous I/O Multiplexing"
///
/// Poll server for input
///
/// Bindings: `nix`
///
/// Protocol: `TCP`
///
/// When a connection is ready-to-read, we'll read the data from it and send that data  to all the
/// other connections, so they can see what the other users typed.
/// TCP Server
///
/// Original: [pollserver.c](https://beej.us/guide/bgnet/examples/pollserver.c)
pub fn pollserver(port: u16) {
    // It seems that using hints with AI_PASSIVE is not possible with nix
    // I wonder if rust enforces this, or if it's a limitation of nix
    // I think it makes sense to be explicit about the ip family
    let unspec = SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, port, 0, 0);
    let socket: nix::sys::socket::SockaddrIn6 = unspec.into();

    // We don't need to loop, because we are not using getaddrinfo, which may return multiple results
    let listener = nix::sys::socket::socket(
        nix::sys::socket::AddressFamily::Inet6,
        nix::sys::socket::SockType::Stream,
        nix::sys::socket::SockFlag::empty(),
        None,
    )
    .expect("Failed to create socket");

    nix::sys::socket::setsockopt(&listener, nix::sys::socket::sockopt::ReuseAddr, &true)
        .expect("Failed to set socket options");

    // In the book, we use a loop to bind to the first address we can
    // And if we fail, we close the socket fd, but in rust, we use "expect" to handle the error
    // and we know that when the program exits, rust will drop the socket fd
    // https://doc.rust-lang.org/std/os/fd/struct.OwnedFd.html
    nix::sys::socket::bind(listener.as_raw_fd(), &socket).expect("Failed to bind to socket");
    let backlog = nix::sys::socket::Backlog::new(10).expect("Failed to create backlog");
    nix::sys::socket::listen(&listener, backlog).expect("Failed to listen on socket");
    // Report ready to read on incoming connection
    let listener_pfd = nix::poll::PollFd::new(listener.as_fd(), nix::poll::PollFlags::POLLIN);

    // We allocate a Vec with 5 connections, but it grows automatically
    let mut pfds = Vec::with_capacity(5);
    pfds.push(listener_pfd);
    println!("Listening on {}", unspec);

    loop {
        println!("polling for events, we have '{}' poll fds", pfds.len());
        // Poll for events
        let num_events =
            nix::poll::poll(&mut pfds, nix::poll::PollTimeout::NONE).expect("poll failed");

        if num_events > 0 {
            println!("Events ready: {}", num_events);
        }
        // Run through all the fds and check if they are ready to read
        for i in 0..pfds.len() {
            let pfd = pfds[i];
            if let Some(e) = pfd.revents() {
                if e.contains(nix::poll::PollFlags::POLLIN) {
                    // If we are the listener socket, handle new connection
                    if pfd.as_fd().as_raw_fd() == listener.as_raw_fd() {
                        println!("[New connection] Attaching to poll list");
                        let new_fd = nix::sys::socket::accept(pfd.as_fd().as_raw_fd())
                            .expect("Failed to accept new connection");
                        let new_bfd = unsafe { BorrowedFd::borrow_raw(new_fd) };
                        let new_pfd = nix::poll::PollFd::new(new_bfd, nix::poll::PollFlags::POLLIN);
                        pfds.push(new_pfd);

                        // convert fd to address
                        let ss: nix::sys::socket::SockaddrStorage =
                            nix::sys::socket::getpeername(new_fd).expect("getpeername failed");
                        println!("New connection, {ss:?}");
                    } else {
                        // Regular client
                        println!("[Client] Preparing to read...");
                        let mut buf = [0u8; 1024];
                        let nbytes = nix::sys::socket::recv(
                            pfd.as_fd().as_raw_fd(),
                            &mut buf,
                            nix::sys::socket::MsgFlags::empty(),
                        )
                        .expect("recv failed");
                        println!("[Client] Received {} bytes", nbytes);

                        // Client closed the connection
                        if nbytes <= 0 {
                            // EOF
                            // Use CTRL+5 and then type "quit" to close the connection
                            // On Mac CTRL + C doesn't work
                            println!("[Client] Connection closed");
                            pfds.remove(i);
                        } else {
                            println!("[Client] Reading bytes...");
                            // Send data to all clients but the sender and listener
                            for j in 1..pfds.len() {
                                if i != j {
                                    let target_pfd = &pfds[j];
                                    let ss: nix::sys::socket::SockaddrStorage =
                                        nix::sys::socket::getpeername(
                                            target_pfd.as_fd().as_raw_fd(),
                                        )
                                        .expect("getpeername failed");
                                    nix::sys::socket::sendto(
                                        target_pfd.as_fd().as_raw_fd(),
                                        &buf[..nbytes],
                                        &ss,
                                        nix::sys::socket::MsgFlags::empty(),
                                    )
                                    .expect("sendto failed");
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
