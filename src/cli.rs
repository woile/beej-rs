use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use clap::{Parser, Subcommand};

use crate::types::Family;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {

    /// Section 5.1 "getaddrinfo() -- Prepare to Launch":
    /// Show IP addresses
    ShowIp {
        /// URL to get IP from
        host: String,

        /// Ipv4 or Ipv6
        #[arg(short, long, value_enum, default_value_t = Family::Unspecified)]
        family: Family,

        /// Service that maps to a port or a port
        #[arg(short, long, default_value = "http")]
        service: String,

    },

    /// Section 6.1 "A Simple Stream Server":
    /// TCP server
    StreamServer,

    /// Section 6.2 "A Simple Stream Client":
    /// TCP client
    StreamClient {
        /// URL to connect to
        host: String,
    },

    /// Section 6.3 "Datagram Sockets":
    /// UDP server.
    /// From now on, we use `nix` for c bindings, which is a little bit safer than `libc`
    SocketListener {
        /// Port used by localhost
        #[arg(short, long, default_value_t = 4950)]
        port: u16,

        #[arg(short, long, default_value_t = Family::Ipv6)]
        family: Family,
    },

    /// Section 6.3 "Datagram Sockets":
    /// UDP client
    SocketTalker {

        /// Host IPv4 or IPv6
        #[arg(long, default_value_t = IpAddr::V6(Ipv6Addr::LOCALHOST))]
        host: IpAddr,

        /// Port used by localhost
        #[arg(short, long, default_value_t = 4950)]
        port: u16,

        /// Message to send
        message: String,
    },

    /// Section 7.2 "poll() - Synchonous I/O Multiplexing":
    /// Poll stdin for input
    PollStdIn,

    /// Section 7.2 "poll() - Synchonous I/O Multiplexing":
    /// Poll server for input
    PollServer {
        /// Port used by localhost
        #[arg(short, long, default_value_t = 9034)]
        port: u16,
    },

    /// Section 7.3 "select()—Synchronous I/O Multiplexing, Old School":
    /// Wait for something to appear on standard input
    Select,

    /// Section 7.3 "select()—Synchronous I/O Multiplexing, Old School":
    /// Simple multi-user chat server
    SelectServer {
        /// Port used by localhost
        #[arg(short, long, default_value_t = 9034)]
        port: u16,
    },

    /// Section 7.7 "Broadcast Packets—Hello, World!":
    /// A UDP Client that broadcasts
    Broadcaster {

        /// Host IPv4 only
        #[arg(long, default_value_t = Ipv4Addr::LOCALHOST)]
        host: Ipv4Addr,

        /// Port used by localhost
        #[arg(short, long, default_value_t = 4950)]
        port: u16,

        /// Message to send
        message: String,
    }
}
