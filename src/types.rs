
use std::fmt::Display;

use clap::ValueEnum;

#[derive(Debug, Clone, PartialEq, ValueEnum)]
pub enum Family {
    Ipv4,
    Ipv6,
    Unspecified,
}

impl Display for Family {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Family::Ipv4 => write!(f, "ipv4"),
            Family::Ipv6 => write!(f, "ipv6"),
            Family::Unspecified => write!(f, "unspecified"),
        }
    }
}

impl Into<libc::c_int> for Family {
    fn into(self) -> libc::c_int {
        match self {
            Family::Ipv4 => libc::AF_INET,
            Family::Ipv6 => libc::AF_INET6,
            Family::Unspecified => libc::AF_UNSPEC,
        }
    }
}

impl From<libc::c_int> for Family {
    fn from(family: libc::c_int) -> Self {
        match family {
            libc::AF_INET => Family::Ipv4,
            libc::AF_INET6 => Family::Ipv6,
            _ => Family::Unspecified,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SocketType {
    Stream,
    Datagram,
}

impl Into<libc::c_int> for SocketType {
    fn into(self) -> libc::c_int {
        match self {
            SocketType::Stream => libc::SOCK_STREAM,
            SocketType::Datagram => libc::SOCK_DGRAM,
        }
    }
}

impl From<libc::c_int> for SocketType {
    fn from(socktype: libc::c_int) -> Self {
        match socktype {
            libc::SOCK_STREAM => SocketType::Stream,
            libc::SOCK_DGRAM => SocketType::Datagram,
            _ => panic!("Unknown socket type"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Flag {
    /// No flags
    None,

    /// assign the address of my local host to the socket structures
    Passive,
}

impl Into<libc::c_int> for Flag {
    fn into(self) -> libc::c_int {
        match self {
            Flag::None => 0,
            Flag::Passive => libc::AI_PASSIVE,
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum SockFd {
    Empty,
    Initialized(i32),
}

impl Into<i32> for SockFd {
    fn into(self) -> i32 {
        match self {
            SockFd::Empty => -1,
            SockFd::Initialized(fd) => fd,
        }
    }
}