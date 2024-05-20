# Beej-rs

> An implementations in rust of the different code samples in [Beej's Guide to Network Programming](https://beej.us/guide/bgnet/)

> [!WARNING]
> The first few exercises were implemented using [libc](https://docs.rs/libc/latest/libc/index.html) which has a lot of unsafe code.
> The latter exercises were impemented using [nix](https://docs.rs/nix/0.28.0/nix/index.html) which is a safer alternative.

## Usage

```console
> cargo run -- --help
Usage: beej-rs <COMMAND>

Commands:
  show-ip          Section 5.1 "getaddrinfo() -- Prepare to Launch": Show IP addresses
  stream-server    Section 6.1 "A Simple Stream Server": TCP server
  stream-client    Section 6.2 "A Simple Stream Client": TCP client
  socket-listener  Section 6.3 "Datagram Sockets": UDP server. From now on, we use `nix` for c bindings, which is a little bit safer than `libc`
  socket-talker    Section 6.3 "Datagram Sockets": UDP client
  poll-std-in      Section 7.2 "poll() - Synchonous I/O Multiplexing": Poll stdin for input
  poll-server      Section 7.2 "poll() - Synchonous I/O Multiplexing": Poll server for input
  select           Section 7.3 "select()—Synchronous I/O Multiplexing, Old School": Wait for something to appear on standard input
  select-server    Section 7.3 "select()—Synchronous I/O Multiplexing, Old School": Simple multi-user chat server
  broadcaster      Section 7.7 "Broadcast Packets—Hello, World!": A UDP Client that broadcasts
  help             Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Examples

- Section 5.1 "getaddrinfo() -- Prepare to Launch"
  - Bindings: `libc`
  - [showip.c](https://beej.us/guide/bgnet/examples/showip.c) -> [showip.rs](./src/examples/showip.rs)
- Section 6.1 "A Simple Stream Server"
  - Bindings: `libc`
  - Protocol: `TCP`
  - [server.c](https://beej.us/guide/bgnet/examples/server.c) -> [server.rs](./src/examples/server.rs)
- Section 6.2 "A Simple Stream Client"
  - Bindings: `libc`
  - Protocol: `TCP`
  - [client.c](https://beej.us/guide/bgnet/examples/client.c) -> [client.rs](./src/examples/client.rs)
- Section 6.3 "Datagram Sockets" UDP Server
  - Bindings: `nix`
  - Protocol: `UDP`
  - [listener.c](https://beej.us/guide/bgnet/examples/listener.c) -> [listener.rs](./src/examples/listener.rs)
- Section 6.3 "Datagram Sockets" UDP Client
  - Bindings: `nix`
  - Protocol: `UDP`
  - [talker.c](https://beej.us/guide/bgnet/examples/talker.c) -> [talker.rs](./src/examples/talker.rs)
- Section 7.2 "poll() - Synchonous I/O Multiplexing"
  - Bindings: `nix`
  - Poll stdin for input
  - [poll.c](https://beej.us/guide/bgnet/examples/poll.c) -> [poll.rs](./src/examples/poll.rs)
- Section 7.2 "poll() - Synchonous I/O Multiplexing"
  - Poll server for input
  - Bindings: `nix`
  - Protocol: `TCP`
  - [pollserver.c](https://beej.us/guide/bgnet/examples/pollserver.c) -> [pollserver.rs](./src/examples/pollserver.rs)
- Section 7.3 "select()—Synchronous I/O Multiplexing, Old School"
  - Bindings: `nix`
  - select from stdin
  - [select.c](<(https://beej.us/guide/bgnet/examples/select.c)>) -> [select.rs](./src/examples/select.rs)
- Section 7.3 "select()—Synchronous I/O Multiplexing, Old School"
  - select server
  - Bindings: `nix`
  - Protocol: `TCP`
  - [selectserver.c](https://beej.us/guide/bgnet/examples/select.c) -> [selectserver.rs](./src/examples/selectserver.rs)
- Section 7.7 "Broadcast Packets—Hello, World!"
  - Bindings: `nix`
  - Protocol: `UDP`
  - broadcast client
  - [broadcaster.c](https://beej.us/guide/bgnet/examples/broadcaster.c) -> [broadcaster.rs](./src/examples/broadcaster.rs)

Note: the design of the cli grew organically as I was reading the book, so it's quite incoherent
