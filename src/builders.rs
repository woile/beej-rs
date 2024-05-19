use std::mem;

use typed_builder::TypedBuilder;
use crate::types::{Family, SocketType, Flag};

#[derive(PartialEq, TypedBuilder)]
pub struct AddrInfo {
    #[builder(default=Family::Unspecified)]
    family: Family,
    #[builder(default=SocketType::Stream)]
    socktype: SocketType,
    #[builder(default=Flag::None)]
    flags: Flag
}

impl Into<libc::addrinfo> for AddrInfo {
    fn into(self) -> libc::addrinfo {
        unsafe {
            let mut addrinfo: libc::addrinfo = mem::zeroed();
            addrinfo.ai_family = self.family.into();
            addrinfo.ai_socktype = self.socktype.into();
            addrinfo.ai_flags = self.flags.into();
            addrinfo
        }
    }
}