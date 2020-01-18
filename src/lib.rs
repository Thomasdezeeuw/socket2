// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// # Source code structure.
//
// All types and methods that are available on (almost) all platforms are
// defined in the first level of the source, i.e. `src/*.rs` files. Additional
// API that is platform specific, e.g. `accept4(2)` is defined in
// `src/sys/*.rs`, only for the platforms that support it.

//! Utilities for creating sockets.
//!
//! The goal of this crate is to create a socket using advanced configuration
//! options (those that are not available in the stdlib types) without using any
//! code.
//!
//! This crate provides **no** cross-platform utilities, no extra goodies, no
//! creature comforts. It is up to the user to known how to use sockets when
//! using this crate. *If you don't know how to create a socket using
//! libc/system calls then this crate is not for you*.
//!
//! To get started see the [`Socket`] type.

#![doc(html_root_url = "https://docs.rs/socket2/0.3")]
#![deny(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    unused_imports,
    dead_code
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
// Disallow warnings when running tests.
#![cfg_attr(test, deny(warnings))]
// Disallow warnings in examples.
#![doc(test(attr(deny(warnings))))]

use std::net::SocketAddr;

mod sockaddr;
mod socket;
mod utils;

#[cfg(unix)]
#[path = "sys/unix.rs"]
mod sys;
#[cfg(windows)]
#[path = "sys/windows.rs"]
mod sys;

use sys::c_int;

pub use sockaddr::SockAddr;
pub use socket::Socket;

/// Specification of the communication domain for a socket.
///
/// This is a newtype wrapper around an integer which provides a nicer API in
/// addition to an injection point for documentation. Convenience constructors
/// such as `Domain::ipv4`, `Domain::ipv6`, etc, are provided to avoid reaching
/// into libc for various constants.
///
/// This type is freely interconvertible with the `i32` type, however, if a raw
/// value needs to be provided.
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Domain(c_int);

impl Domain {
    /// Domain for IPv4 communication, corresponding to `AF_INET`.
    pub const IPV4: Domain = Domain(sys::AF_INET);

    /// Domain for IPv6 communication, corresponding to `AF_INET6`.
    pub const IPV6: Domain = Domain(sys::AF_INET6);

    /// Returns the correct `Domain` for the `addr`.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::net::SocketAddr;
    /// use socket2::Domain;
    ///
    /// let addr: SocketAddr = "127.0.0.1:1234".parse().unwrap();
    /// let domain = Domain::for_ip(&addr);
    /// assert_eq!(domain, Domain::IPV4);
    /// ```
    pub fn for_ip(addr: &SocketAddr) -> Domain {
        match addr {
            SocketAddr::V4(..) => Domain::IPV4,
            SocketAddr::V6(..) => Domain::IPV6,
        }
    }
}

impl From<c_int> for Domain {
    fn from(d: c_int) -> Domain {
        Domain(d)
    }
}

impl From<Domain> for c_int {
    fn from(d: Domain) -> c_int {
        d.0
    }
}

/// Specification of communication semantics on a socket.
///
/// This is a newtype wrapper around an integer which provides a nicer API in
/// addition to an injection point for documentation. Convenience constructors
/// such as `Type::stream`, `Type::dgram`, etc, are provided to avoid reaching
/// into libc for various constants.
///
/// This type is freely interconvertible with the `i32` type, however, if a raw
/// value needs to be provided.
#[derive(Copy, Clone, Debug)]
pub struct Type(c_int);

impl Type {
    /// Type corresponding to `SOCK_STREAM`.
    ///
    /// Used for protocols such as TCP.
    pub fn stream() -> Type {
        Type(sys::SOCK_STREAM)
    }

    /// Type corresponding to `SOCK_DGRAM`.
    ///
    /// Used for protocols such as UDP.
    pub fn dgram() -> Type {
        Type(sys::SOCK_DGRAM)
    }

    /// Type corresponding to `SOCK_SEQPACKET`.
    pub fn seqpacket() -> Type {
        Type(sys::SOCK_SEQPACKET)
    }

    /// Type corresponding to `SOCK_RAW`.
    #[cfg(not(target_os = "redox"))]
    pub fn raw() -> Type {
        Type(sys::SOCK_RAW)
    }
}

impl From<c_int> for Type {
    fn from(t: c_int) -> Type {
        Type(t)
    }
}

impl From<Type> for c_int {
    fn from(t: Type) -> c_int {
        t.0
    }
}

/// Protocol specification used for creating sockets via `Socket::new`.
///
/// This is a newtype wrapper around an integer which provides a nicer API in
/// addition to an injection point for documentation.
///
/// This type is freely interconvertible with the `i32` type, however, if a raw
/// value needs to be provided.
#[derive(Copy, Clone, Debug)]
pub struct Protocol(c_int);

impl Protocol {
    /// Protocol corresponding to `ICMPv4`.
    pub fn icmpv4() -> Self {
        Protocol(sys::IPPROTO_ICMP)
    }

    /// Protocol corresponding to `ICMPv6`.
    pub fn icmpv6() -> Self {
        Protocol(sys::IPPROTO_ICMPV6)
    }

    /// Protocol corresponding to `TCP`.
    pub fn tcp() -> Self {
        Protocol(sys::IPPROTO_TCP)
    }

    /// Protocol corresponding to `UDP`.
    pub fn udp() -> Self {
        Protocol(sys::IPPROTO_UDP)
    }
}

impl From<c_int> for Protocol {
    fn from(p: c_int) -> Protocol {
        Protocol(p)
    }
}

impl From<Protocol> for c_int {
    fn from(p: Protocol) -> c_int {
        p.0
    }
}
