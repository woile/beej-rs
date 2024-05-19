use std::{
    ffi::{CStr, CString},
    io::stdin,
    mem,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    os::fd::{AsFd, AsRawFd, BorrowedFd, RawFd},
    ptr,
};

use beej_rs::{
    builders::AddrInfo,
    cli::{Cli, Commands},
    types::{Family, Flag, SockFd, SocketType},
};

use clap::Parser;
use socket2::SockAddr;

/// Section 5.1 "getaddrinfo() -- Prepare to Launch"
fn showip(host: String, family: Family, service: String) {
    println!("IP addresses for '{}'\n", host);

    let host = CString::new(host).expect("Invalid host");
    let c_host: *const libc::c_char = host.as_ptr() as *const libc::c_char;

    let service =
        CString::new(service).expect("Invalid service, service should map to a port number");
    let service: *const libc::c_char = service.as_ptr() as *const libc::c_char;
    let addrino = AddrInfo::builder().family(family).build();

    let hints = addrino.into();

    let mut res = ptr::null_mut();
    unsafe { libc::getaddrinfo(c_host, service, &hints, &mut res) };

    while !res.is_null() {
        let ((), sockaddr) = unsafe {
            SockAddr::try_init(|storage, len| {
                *len = (*res).ai_addrlen as _;
                std::ptr::copy_nonoverlapping(
                    (*res).ai_addr as *const u8,
                    storage as *mut u8,
                    (*res).ai_addrlen as usize,
                );
                Ok(())
            })
        }
        .expect("to create a socket");

        res = (unsafe { *res }).ai_next as *mut libc::addrinfo;
        println!("\t{}", sockaddr.as_socket().expect("failed to extract IP"));

        match sockaddr.family() as i32 {
            libc::AF_INET => {
                println!("\tFamily: IPv4");
            }
            libc::AF_INET6 => {
                println!("\tFamily: IPv6");
            }
            _ => {
                println!("\tUnknown family");
            }
        }
    }
}

// helper function
fn showaddrinfo(addr: &libc::addrinfo) {
    let ip = unsafe {
        match (*addr).ai_family {
            libc::AF_INET => {
                let addr = (*addr).ai_addr as *const libc::sockaddr_in;
                let addr = &*addr;
                let ip = addr.sin_addr;
                let ip = ip.s_addr;
                let ip = ip.to_be();
                let ip = IpAddr::V4(ip.into());
                ip
            }
            libc::AF_INET6 => {
                let addr = (*addr).ai_addr as *const libc::sockaddr_in6;
                let addr = &*addr;
                let ip = addr.sin6_addr;
                let ip = ip.s6_addr;
                let ip = IpAddr::V6(ip.into());
                ip
            }
            _ => {
                panic!("Unknown family");
            }
        }
    };
    println!("IP: {:?}", ip);
}

/// Section 6.1 "A Simple Stream Server"
///
/// Bindings: `libc`
///
/// Protocol: `TCP`
///
/// Original: [server.c](https://beej.us/guide/bgnet/examples/server.c)
fn streamserver() {
    let family = Family::Unspecified;
    let host = "localhost";
    let service = "3490";

    // let addrinfo = unsafe { buildaddrinfo(Family::Unspecified) };
    let addrinfo = AddrInfo::builder()
        .family(family)
        .flags(Flag::Passive)
        .build();
    let hints: libc::addrinfo = addrinfo.into();

    let mut servinfo = ptr::null_mut();
    let rv = unsafe {
        let node = CString::new(host).expect("Invalid node");
        let c_node: *const libc::c_char = node.as_ptr() as *const libc::c_char;
        let port = CString::new(service).expect("Invalid port");
        let c_port: *const libc::c_char = port.as_ptr() as *const libc::c_char;
        // let c_port = port.as_ptr();
        println!("Starting server in {host}:{service}");
        libc::getaddrinfo(c_node, c_port, &hints, &mut servinfo)
    };

    if rv != 0 {
        eprintln!("getaddrinfo: {}", unsafe {
            CStr::from_ptr(libc::gai_strerror(rv)).to_str().unwrap()
        });
        return;
    }

    let mut sockfd = SockFd::Empty;
    // loop through all the results and bind to the first we can
    while !servinfo.is_null() {
        unsafe {
            let _sockfd = libc::socket(
                (*servinfo).ai_family,
                (*servinfo).ai_socktype,
                (*servinfo).ai_protocol,
            );
            if _sockfd == -1 {
                eprintln!("server: socket err");
                servinfo = (*servinfo).ai_next as *mut libc::addrinfo;
                continue;
            }
            let optval_yes: libc::c_int = 1;
            let errr = libc::setsockopt(
                _sockfd,
                libc::SOL_SOCKET,
                libc::SO_REUSEADDR,
                &optval_yes as *const _ as *const libc::c_void,
                mem::size_of_val(&optval_yes) as libc::socklen_t,
            );

            if errr == -1 {
                eprintln!("server: setsockopt err");
                libc::exit(1);
            }
            let errr = libc::bind(
                _sockfd,
                (*servinfo).ai_addr,
                (*servinfo).ai_addrlen as libc::socklen_t,
            );
            if errr == -1 {
                libc::close(_sockfd);
                eprintln!("server: bind err");
                servinfo = (*servinfo).ai_next as *mut libc::addrinfo;
                continue;
            }
            sockfd = SockFd::Initialized(_sockfd);
        }
        break;
    }

    if servinfo.is_null() {
        eprintln!("server: failed to bind socket");
        unsafe { libc::exit(1) };
    }

    if sockfd == SockFd::Empty {
        eprintln!("server: failed to create socket");
        unsafe { libc::exit(1) };
    }
    let sockfd = sockfd.into();
    let errr = unsafe {
        // how many pending connections queue will hold
        let backlog = 10;
        libc::listen(sockfd, backlog)
    };

    if errr == -1 {
        eprintln!("server: listen err");
        unsafe { libc::exit(1) };
    }

    println!("server: waiting for connections...");

    loop {
        let mut their_addr = mem::MaybeUninit::<libc::sockaddr_storage>::uninit();
        let mut sin_size = mem::size_of::<libc::sockaddr_storage>() as libc::socklen_t;
        let new_fd = unsafe {
            libc::accept(
                sockfd,
                their_addr.as_mut_ptr() as *mut libc::sockaddr,
                &mut sin_size,
            )
        };

        if new_fd == -1 {
            eprintln!("server: accept err");
            continue;
        }

        let s = unsafe { SockAddr::new(their_addr.assume_init(), sin_size) };
        println!("server: got connection from {:?}", s);

        let msg = CString::new("Hello, world!").expect("Invalid message");
        let len = msg.as_bytes().len();
        let errr = unsafe {
            libc::send(
                new_fd,
                msg.as_ptr() as *const libc::c_void,
                len,
                libc::MSG_NOSIGNAL,
            )
        };

        if errr == -1 {
            eprintln!("server: send err");
        }

        unsafe { libc::close(new_fd) };
    }
}

/// Section 6.2 "A Simple Stream Client"
/// Bindings: `libc`
///
/// Protocol: `TCP`
///
/// Original: [client.c](https://beej.us/guide/bgnet/examples/client.c)
fn streamclient(host: String) {
    let service = "3490";

    let hints = AddrInfo::builder()
        .family(Family::Unspecified)
        .socktype(SocketType::Stream)
        .build();

    let mut servinfo = ptr::null_mut();
    unsafe {
        let node = CString::new(host).expect("Invalid host");
        let c_node: *const libc::c_char = node.as_ptr() as *const libc::c_char;
        let service = CString::new(service).expect("Invalid service");
        let c_service: *const libc::c_char = service.as_ptr() as *const libc::c_char;
        let hints = hints.into();
        libc::getaddrinfo(c_node, c_service, &hints, &mut servinfo);
    }

    let mut sockfd = mem::MaybeUninit::<libc::c_int>::uninit();
    while !servinfo.is_null() {
        unsafe {
            let _sockfd = libc::socket(
                (*servinfo).ai_family,
                (*servinfo).ai_socktype,
                (*servinfo).ai_protocol,
            );

            if _sockfd == -1 {
                servinfo = (*servinfo).ai_next as *mut libc::addrinfo;
                continue;
            }

            let errr = libc::connect(_sockfd, (*servinfo).ai_addr, (*servinfo).ai_addrlen);
            if errr == -1 {
                libc::close(_sockfd);
                servinfo = (*servinfo).ai_next as *mut libc::addrinfo;
                eprintln!("client: connect err");
                continue;
            }
            sockfd.write(_sockfd);
        }
        break;
    }

    if servinfo.is_null() {
        eprintln!("client: failed to connect");
        unsafe { libc::exit(2) };
    }
    showaddrinfo(unsafe { servinfo.as_ref().unwrap() });
    const MAXDATASIZE: usize = 100;
    let mut buf = [0u8; MAXDATASIZE];
    let numbytes = unsafe {
        libc::recv(
            sockfd.assume_init(),
            buf.as_mut_ptr() as *mut libc::c_void,
            MAXDATASIZE - 1,
            0,
        )
    };
    if numbytes == -1 {
        eprintln!("client: recv err");
        unsafe { libc::exit(1) };
    }
    println!("client: received '{}'", unsafe {
        CStr::from_ptr(buf.as_ptr() as *const libc::c_char)
            .to_str()
            .unwrap()
    });
}

/// Section 6.3 "Datagram Sockets"
///
/// Bindings: `nix`
///
/// Protocol: `UDP`
///
/// This is a UDP server listening to UDP messages.
/// Because UDP is connectionless and fires packets off, we are explicit about the family (ipv4 or ipv6)
///
/// Original: [listener.c](https://beej.us/guide/bgnet/examples/listener.c)
fn socketlistener(port: u16, family: Family) {
    let local_addr = match family {
        Family::Ipv4 => SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, port)),
        Family::Ipv6 | Family::Unspecified => {
            SocketAddr::V6(SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, port, 0, 0))
        }
    };

    let sockfd = match local_addr {
        SocketAddr::V4(localhost) => {
            let socket: nix::sys::socket::SockaddrIn = localhost.into();

            // give me a FD for a socket
            let sockfd = nix::sys::socket::socket(
                nix::sys::socket::AddressFamily::Inet,
                nix::sys::socket::SockType::Datagram,
                nix::sys::socket::SockFlag::empty(),
                None,
            )
            .expect("Failed to create socket");

            // bind the socket to the address
            nix::sys::socket::bind(sockfd.as_raw_fd(), &socket).expect("Failed to bind to socket");

            let ss: nix::sys::socket::SockaddrStorage =
                nix::sys::socket::getsockname(sockfd.as_raw_fd()).expect("getsockname");
            println!(
                "Listening on {}",
                ss.as_sockaddr_in().expect("sockaddr not ipv4")
            );

            sockfd
        }
        SocketAddr::V6(localhost) => {
            let socket: nix::sys::socket::SockaddrIn6 = localhost.into();

            // give me a FD for a socket
            let sockfd = nix::sys::socket::socket(
                nix::sys::socket::AddressFamily::Inet6,
                nix::sys::socket::SockType::Datagram,
                nix::sys::socket::SockFlag::empty(),
                None,
            )
            .expect("Failed to create socket");

            // bind the socket to the address
            nix::sys::socket::bind(sockfd.as_raw_fd(), &socket).expect("Failed to bind to socket");

            let ss: nix::sys::socket::SockaddrStorage =
                nix::sys::socket::getsockname(sockfd.as_raw_fd()).expect("getsockname");
            println!(
                "Listening on {}",
                ss.as_sockaddr_in6().expect("sockaddr not ipv6")
            );

            sockfd
        }
    };

    // This loop is not in the book, I just added it to avoid
    // having to run the program multiple times
    loop {
        let mut buf = [0u8; 1024];
        let (len, _addr) = nix::sys::socket::recvfrom::<nix::sys::socket::SockaddrStorage>(
            sockfd.as_raw_fd(),
            &mut buf,
        )
        .expect("recvfrom failed");
        // println!("Received {} bytes from {:?}", len, addr);
        let msg = std::str::from_utf8(&buf[..len]).expect("Failed to convert to string");
        println!("{}", msg);
    }
}

/// Section 6.3 "Datagram Sockets"
///
/// UDP Client
///
/// Bindings: `nix`
///
/// Protocol: `UDP`
///
/// We are explicit about the family (IPv4 or Ipv6).
/// Remember: data sent over UDP is not guaranteed to arrive,
/// you can run the talker without a server, and the messages
/// will be lost.
///
/// Original: [talker.c](https://beej.us/guide/bgnet/examples/talker.c)
fn sockettalker(host: IpAddr, port: u16, message: String) {
    match host {
        IpAddr::V4(addr) => {
            let socket = SocketAddrV4::new(addr, port);
            let socket: nix::sys::socket::SockaddrIn = socket.into();
            let sockfd = nix::sys::socket::socket(
                nix::sys::socket::AddressFamily::Inet,
                nix::sys::socket::SockType::Datagram,
                nix::sys::socket::SockFlag::empty(),
                None,
            )
            .expect("Failed to create sockfd");

            nix::sys::socket::sendto(
                sockfd.as_raw_fd(),
                message.as_bytes(),
                &socket,
                nix::sys::socket::MsgFlags::empty(),
            )
            .expect("Failed to send message");
        }
        IpAddr::V6(addr) => {
            let socket = SocketAddrV6::new(addr, port, 0, 0);
            let socket: nix::sys::socket::SockaddrIn6 = socket.into();
            let sockfd = nix::sys::socket::socket(
                nix::sys::socket::AddressFamily::Inet6,
                nix::sys::socket::SockType::Datagram,
                nix::sys::socket::SockFlag::empty(),
                None,
            )
            .expect("Failed to create sockfd");

            nix::sys::socket::sendto(
                sockfd.as_raw_fd(),
                message.as_bytes(),
                &socket,
                nix::sys::socket::MsgFlags::empty(),
            )
            .expect("Failed to send message");
        }
    };
}

/// Section 7.2 "poll() - Synchonous I/O Multiplexing"
///
/// Bindings: `nix`
///
/// Ask the OS to do the dirty work of polling for us,
/// to avoid consuming too much CPU time.
/// When polling, the process is put to sleep until the FD is ready.
///
/// Original: [poll.c](https://beej.us/guide/bgnet/examples/poll.c)
fn pollstdin() {
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
fn pollserver(port: u16) {
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

/// Section 7.3 "select()—Synchronous I/O Multiplexing, Old School"
///
/// Bindings: `nix`
///
/// Wait for something to appear on standard input.
/// When running you need to write something and hit ENTER.
///
/// Original: [select.c]((https://beej.us/guide/bgnet/examples/select.c))
fn select() {
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
fn select_server(port: u16) {
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

/// Section 7.7 "Broadcast Packets—Hello, World!"
///
/// Broadcast client
///
/// IPv4 Only
///
/// Bindings: `nix`
///
/// Protocol: `UDP`
///
///
/// Original: [broadcaster.c](https://beej.us/guide/bgnet/examples/broadcaster.c)
fn broadcaster(host: Ipv4Addr, port: u16, message: String) {
    let localhost = SocketAddrV4::new(host, port);
    let socket: nix::sys::socket::SockaddrIn = localhost.into();
    let sockfd = nix::sys::socket::socket(
        nix::sys::socket::AddressFamily::Inet,
        nix::sys::socket::SockType::Datagram,
        nix::sys::socket::SockFlag::empty(),
        None,
    )
    .expect("Failed to create sockfd");

    // Set the socket as Broadcast
    nix::sys::socket::setsockopt(&sockfd, nix::sys::socket::sockopt::Broadcast, &true)
        .expect("Failed to set socket options");
    println!("Sending message to {}", localhost);
    nix::sys::socket::sendto(
        sockfd.as_raw_fd(),
        message.as_bytes(),
        &socket,
        nix::sys::socket::MsgFlags::empty(),
    )
    .expect("Failed to send message");
}
fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::ShowIp {
            host,
            family,
            service,
        } => showip(host, family, service),
        Commands::StreamServer => streamserver(),
        Commands::StreamClient { host } => streamclient(host),
        Commands::SocketListener { port, family } => socketlistener(port, family),
        Commands::SocketTalker {
            host,
            port,
            message,
        } => sockettalker(host, port, message),
        Commands::PollStdIn => pollstdin(),
        Commands::PollServer { port } => pollserver(port),
        Commands::Select => select(),
        Commands::SelectServer { port } => select_server(port),
        Commands::Broadcaster {
            host,
            port,
            message,
        } => broadcaster(host, port, message),
    }
}
