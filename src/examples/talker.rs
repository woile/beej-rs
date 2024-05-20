use std::{
    net::{IpAddr, SocketAddrV4, SocketAddrV6},
    os::fd::AsRawFd,
};

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
pub fn sockettalker(host: IpAddr, port: u16, message: String) {
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
