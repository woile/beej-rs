use std::{
    ffi::{CStr, CString},
    mem, ptr,
};

use socket2::SockAddr;

use crate::{
    builders::AddrInfo,
    types::{Family, Flag, SockFd},
};

/// Section 6.1 "A Simple Stream Server"
///
/// Contains unsafe code
///
/// Bindings: `libc`
///
/// Protocol: `TCP`
///
/// Original: [server.c](https://beej.us/guide/bgnet/examples/server.c)
pub fn streamserver() {
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
