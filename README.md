# Beej-rs

> An implementations in rust of the different code samples in [Beej's Guide to Network Programming](https://beej.us/guide/bgnet/)

## Examples

- Section 5.1 "getaddrinfo() -- Prepare to Launch"
    - Bindings: `libc`
    - [showip.c](https://beej.us/guide/bgnet/examples/showip.c)
- Section 6.1 "A Simple Stream Server"
    - Bindings: `libc`
    - Protocol: `TCP`
    - [server.c](https://beej.us/guide/bgnet/examples/server.c)
- Section 6.2 "A Simple Stream Client"
    - Bindings: `libc`
    - Protocol: `TCP`
    - [client.c](https://beej.us/guide/bgnet/examples/client.c)
- Section 6.3 "Datagram Sockets" UDP Server
    - Bindings: `nix`
    - Protocol: `UDP`
    - [listener.c](https://beej.us/guide/bgnet/examples/listener.c)
- Section 6.3 "Datagram Sockets" UDP Client
    - Bindings: `nix`
    - Protocol: `UDP`
    - [talker.c](https://beej.us/guide/bgnet/examples/talker.c)
- Section 7.2 "poll() - Synchonous I/O Multiplexing"
    - Bindings: `nix`
    - Poll stdin for input
    - [poll.c](https://beej.us/guide/bgnet/examples/poll.c)
- Section 7.2 "poll() - Synchonous I/O Multiplexing"
    - Poll server for input
    - Bindings: `nix`
    - Protocol: `TCP`
    - [pollserver.c](https://beej.us/guide/bgnet/examples/pollserver.c)
- Section 7.3 "select()—Synchronous I/O Multiplexing, Old School"
    - Bindings: `nix`
    - select from stdin
    - [select.c]((https://beej.us/guide/bgnet/examples/select.c))
- Section 7.3 "select()—Synchronous I/O Multiplexing, Old School"
    - select server
    - Bindings: `nix`
    - Protocol: `TCP`
    - [selectserver.c](https://beej.us/guide/bgnet/examples/select.c)
- Section 7.7 "Broadcast Packets—Hello, World!"
    - Bindings: `nix`
    - Protocol: `UDP`
    - broadcast client
    - [broadcaster.c](https://beej.us/guide/bgnet/examples/broadcaster.c)

Note: the design of the cli grew organically as I was reading the book, so it's quite incoherent