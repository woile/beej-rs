use std::{ffi::CString, ptr};

use socket2::SockAddr;

use crate::{builders::AddrInfo, types::Family};

/// Section 5.1 "getaddrinfo() -- Prepare to Launch"
pub fn showip(host: String, family: Family, service: String) {
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
