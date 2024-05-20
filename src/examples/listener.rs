use std::{
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    os::fd::AsRawFd,
};

use crate::types::Family;

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
pub fn socketlistener(port: u16, family: Family) {
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
