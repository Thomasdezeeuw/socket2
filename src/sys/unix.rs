// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io;
use std::mem::{self, size_of, MaybeUninit};
use std::net::Shutdown;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
use std::os::unix::net::{UnixDatagram, UnixListener, UnixStream};

use crate::{Domain, Protocol, SockAddr, Socket, Type};

// Used in conversions for `Domain`, `Type` and `Protocol`.
#[allow(non_camel_case_types)]
pub(crate) type c_int = libc::c_int;

// Used in `Domain`.
pub(crate) use libc::{AF_INET, AF_INET6};
// Used in `Type`.
pub(crate) use libc::{SOCK_DGRAM, SOCK_RAW, SOCK_SEQPACKET, SOCK_STREAM};
// Used in `Protocol`.
pub(crate) use libc::{IPPROTO_ICMP, IPPROTO_ICMPV6, IPPROTO_TCP, IPPROTO_UDP};
// Used in `Socket`.
pub(crate) use std::os::unix::io::RawFd as RawSocket;

/// Unix only API.
impl Domain {
    /// Domain for Unix socket communication, corresponding to `AF_UNIX`.
    pub const UNIX: Domain = Domain(libc::AF_UNIX);

    /// Domain for low-level packet interface, corresponding to `AF_PACKET`.
    ///
    /// # Notes
    ///
    /// This function is only available on Linux.
    #[cfg(target_os = "linux")]
    pub const PACKET: Domain = Domain(libc::AF_PACKET);
}

/// Unix only API.
impl Type {
    /// Set `SOCK_NONBLOCK` on the `Type`.
    ///
    /// # Notes
    ///
    /// This function is only available on Android, DragonFlyBSD, FreeBSD,
    /// Linux, NetBSD and OpenBSD.
    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    pub fn non_blocking(self) -> Type {
        Type(self.0 | libc::SOCK_NONBLOCK)
    }

    /// Set `SOCK_CLOEXEC` on the `Type`.
    ///
    /// # Notes
    ///
    /// This function is only available on Android, DragonFlyBSD, FreeBSD,
    /// Linux, NetBSD and OpenBSD.
    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    pub fn cloexec(self) -> Type {
        Type(self.0 | libc::SOCK_CLOEXEC)
    }
}

/// Helper macro to execute a system call that returns an `io::Result`.
macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        let res = unsafe { libc::$fn($($arg, )*) };
        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

pub(crate) fn socket(domain: c_int, type_: c_int, protocol: c_int) -> io::Result<Socket> {
    syscall!(socket(domain, type_, protocol)).map(|fd| Socket { inner: fd })
}

pub(crate) fn connect(
    sockfd: RawSocket,
    addr: *const libc::sockaddr_storage,
    addrlen: libc::socklen_t,
) -> io::Result<()> {
    // Most OSes don't actually use `sockaddr_storage` in the `connect(2)` call,
    // but `sockaddr_storage` can be converted safely into the correct type.
    syscall!(connect(sockfd, addr as *const _, addrlen)).map(|_| ())
}

pub(crate) fn bind(
    sockfd: RawSocket,
    addr: *const libc::sockaddr_storage,
    addrlen: libc::socklen_t,
) -> io::Result<()> {
    // Most OSes don't actually use `sockaddr_storage` in the `bind(2)` call,
    // but `sockaddr_storage` can be converted safely into the correct type.
    syscall!(bind(sockfd, addr as *const _, addrlen)).map(|_| ())
}

pub(crate) fn listen(sockfd: RawSocket, backlog: c_int) -> io::Result<()> {
    syscall!(listen(sockfd, backlog)).map(|_| ())
}

pub(crate) fn accept(sockfd: RawSocket) -> io::Result<(Socket, SockAddr)> {
    let mut addr: MaybeUninit<libc::sockaddr_storage> = MaybeUninit::uninit();
    let mut addrlen = size_of::<libc::sockaddr_storage>() as libc::socklen_t;
    syscall!(accept(sockfd, addr.as_mut_ptr() as *mut _, &mut addrlen)).map(|stream_fd| {
        // This is safe because `accept(2)` filled in the address for us.
        let addr = unsafe { SockAddr::from_raw_parts(addr.assume_init(), addrlen) };
        (Socket { inner: stream_fd }, addr)
    })
}

pub(crate) fn getsockname(sockfd: RawSocket) -> io::Result<SockAddr> {
    let mut addr: MaybeUninit<libc::sockaddr_storage> = MaybeUninit::uninit();
    let mut addrlen = size_of::<libc::sockaddr_storage>() as libc::socklen_t;
    syscall!(getsockname(
        sockfd,
        addr.as_mut_ptr() as *mut _,
        &mut addrlen
    ))
    .map(|_| {
        // This is safe because `getsockname(2)` filled in the address for us.
        unsafe { SockAddr::from_raw_parts(addr.assume_init(), addrlen) }
    })
}

pub(crate) fn getpeername(sockfd: RawSocket) -> io::Result<SockAddr> {
    let mut addr: MaybeUninit<libc::sockaddr_storage> = MaybeUninit::uninit();
    let mut addrlen = size_of::<libc::sockaddr_storage>() as libc::socklen_t;
    syscall!(getpeername(
        sockfd,
        addr.as_mut_ptr() as *mut _,
        &mut addrlen
    ))
    .map(|_| {
        // This is safe because `getpeername(2)` filled in the address for us.
        unsafe { SockAddr::from_raw_parts(addr.assume_init(), addrlen) }
    })
}

pub(crate) fn shutdown(sockfd: RawSocket, how: Shutdown) -> io::Result<()> {
    let how = match how {
        Shutdown::Write => libc::SHUT_WR,
        Shutdown::Read => libc::SHUT_RD,
        Shutdown::Both => libc::SHUT_RDWR,
    };
    syscall!(shutdown(sockfd, how)).map(|_| ())
}

pub(crate) fn setsockopt<T>(
    sockfd: RawSocket,
    level: c_int,
    optname: c_int,
    opt: &T,
) -> io::Result<()> {
    syscall!(setsockopt(
        sockfd,
        level,
        optname,
        opt as *const _ as *const _,
        size_of::<T>() as libc::socklen_t,
    ))
    .map(|_| ())
}

pub(crate) fn getsockopt<T>(sockfd: RawSocket, level: c_int, optname: c_int) -> io::Result<T> {
    let mut optval: MaybeUninit<T> = MaybeUninit::uninit();
    let mut optlen = size_of::<T>() as libc::socklen_t;
    syscall!(getsockopt(
        sockfd,
        level,
        optname,
        optval.as_mut_ptr() as *mut _,
        &mut optlen
    ))
    .map(|_| unsafe {
        // Safe because `getsockopt(2)` initialised the value for us.
        debug_assert_eq!(optlen as usize, size_of::<T>());
        optval.assume_init()
    })
}

pub(crate) fn fcntl<T>(sockfd: RawSocket, cmd: c_int, arg: T) -> io::Result<c_int> {
    syscall!(fcntl(sockfd, cmd, arg))
}

/// Unix only API.
impl Socket {
    /// Creates a pair of sockets which are connected to each other.
    ///
    /// This function corresponds to `socketpair(2)`.
    pub fn pair(
        domain: Domain,
        type_: Type,
        protocol: Option<Protocol>,
    ) -> io::Result<(Socket, Socket)> {
        let mut fds = [0, 0];
        let protocol = protocol.map(|p| p.0).unwrap_or(0);
        syscall!(socketpair(domain.0, type_.0, protocol, fds.as_mut_ptr()))
            .map(|_| (Socket { inner: fds[0] }, Socket { inner: fds[1] }))
    }

    /// Accept a new incoming connection from this listener.
    ///
    /// This function directly corresponds to the `accept4(2)` function.
    ///
    /// # Notes
    ///
    /// This only available on Android, DragonFlyBSD, FreeBSD, Linux and
    /// OpenBSD. Once https://github.com/rust-lang/libc/issues/1636 is fixed
    /// NetBSD will also support it.
    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "linux",
        // NetBSD 8.0 actually has `accept4(2)`, but libc doesn't expose it
        // (yet). See https://github.com/rust-lang/libc/issues/1636.
        //target_os = "netbsd",
        target_os = "openbsd"
    ))]
    pub fn accept4(&self, flags: c_int) -> io::Result<(Socket, SockAddr)> {
        let mut addr: MaybeUninit<libc::sockaddr_storage> = MaybeUninit::uninit();
        let mut addrlen = size_of::<libc::sockaddr_storage>() as libc::socklen_t;
        syscall!(accept4(
            self.inner,
            addr.as_mut_ptr() as *mut _,
            &mut addrlen,
            flags
        ))
        .map(|stream_fd| {
            // This is safe because `accept(2)` filled in the address for us.
            let addr = unsafe { SockAddr::from_raw_parts(addr.assume_init(), addrlen) };
            (Socket { inner: stream_fd }, addr)
        })
    }
}

impl From<UnixStream> for Socket {
    fn from(socket: UnixStream) -> Socket {
        unsafe { Socket::from_raw_fd(socket.into_raw_fd()) }
    }
}

impl Into<UnixStream> for Socket {
    fn into(self) -> UnixStream {
        unsafe { UnixStream::from_raw_fd(self.into_raw_fd()) }
    }
}

impl From<UnixListener> for Socket {
    fn from(socket: UnixListener) -> Socket {
        unsafe { Socket::from_raw_fd(socket.into_raw_fd()) }
    }
}

impl Into<UnixListener> for Socket {
    fn into(self) -> UnixListener {
        unsafe { UnixListener::from_raw_fd(self.into_raw_fd()) }
    }
}

impl From<UnixDatagram> for Socket {
    fn from(socket: UnixDatagram) -> Socket {
        unsafe { Socket::from_raw_fd(socket.into_raw_fd()) }
    }
}

impl Into<UnixDatagram> for Socket {
    fn into(self) -> UnixDatagram {
        unsafe { UnixDatagram::from_raw_fd(self.into_raw_fd()) }
    }
}

impl FromRawFd for Socket {
    unsafe fn from_raw_fd(fd: RawFd) -> Socket {
        Socket { inner: fd }
    }
}

impl AsRawFd for Socket {
    fn as_raw_fd(&self) -> RawFd {
        self.inner
    }
}

impl IntoRawFd for Socket {
    fn into_raw_fd(self) -> RawFd {
        let fd = self.inner;
        mem::forget(self);
        fd
    }
}

impl Drop for Socket {
    fn drop(&mut self) {
        // Can't handle the error here, nor can we do much with it.
        let _ = unsafe { libc::close(self.inner) };
    }
}
