use std::{
    net::{Ipv4Addr, SocketAddrV4},
    os::fd::AsRawFd,
};

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
pub fn broadcaster(host: Ipv4Addr, port: u16, message: String) {
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
