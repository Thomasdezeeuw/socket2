// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::net::{Shutdown, TcpListener, TcpStream, UdpSocket};
#[cfg(unix)]
use std::os::unix::io::{FromRawFd, IntoRawFd};
use std::{fmt, io};

use crate::sys::{self, c_int};
use crate::{Domain, Protocol, SockAddr, Type};

/// An owned system socket.
///
/// This type simply wraps an instance of a file descriptor (`int`) on Unix and
/// an instance of `SOCKET` on Windows. This is the main type exported by this
/// crate and is intended to mirror the raw semantics of sockets on platforms as
/// closely as possible. All methods correspond to precisely one libc or OS API
/// call which is essentially just a "Rustic translation" of what's below.
///
/// # Notes
///
/// This type can be converted to and from all network types provided by the
/// standard library using the [`From`] and [`Into`] traits. Is up to the user
/// to ensure the socket is setup correctly for a given type!
///
/// # Examples
///
/// ```
/// # fn main() -> std::io::Result<()> {
/// use std::net::{SocketAddr, TcpListener};
/// use socket2::{Socket, Domain, Type};
///
/// // Create a new `Socket`.
/// let socket = Socket::new(Domain::IPV4, Type::stream(), None)?;
///
/// // Bind the socket to an addresses.
/// let addr1: SocketAddr = "127.0.0.1:15123".parse().unwrap();
/// socket.bind(&addr1.into())?;
///
/// // Start listening on the socket.
/// socket.listen(128)?;
///
/// // Finally convert it to `TcpListener` from the standard library. Now it can
/// // be used like any other `TcpListener`.
/// let listener: TcpListener = socket.into();
/// # drop(listener);
/// # Ok(())
/// # }
/// ```
pub struct Socket {
    // The `sys` module must have access to the raw socket to implement OS
    // specific additional methods, e.g. Unix Domain sockets (UDS).
    pub(crate) inner: sys::RawSocket,
}

impl Socket {
    /// Creates a new socket ready to be configured.
    ///
    /// This function corresponds to `socket(2)`.
    pub fn new(domain: Domain, type_: Type, protocol: Option<Protocol>) -> io::Result<Socket> {
        sys::socket(domain.0, type_.0, protocol.map(|p| p.0).unwrap_or(0))
    }

    /// Initiate a connection on this socket to the specified address.
    ///
    /// This function directly corresponds to the `connect(2)` function.
    pub fn connect(&self, addr: &SockAddr) -> io::Result<()> {
        sys::connect(self.inner, addr.as_ptr(), addr.len())
    }

    /// Binds this socket to the specified address.
    ///
    /// This function directly corresponds to the `bind(2)` function.
    pub fn bind(&self, addr: &SockAddr) -> io::Result<()> {
        sys::bind(self.inner, addr.as_ptr(), addr.len())
    }

    /// Returns the socket address of the local half of this connection.
    ///
    /// This function directly corresponds to the `getsockname(2)` function.
    pub fn local_addr(&self) -> io::Result<SockAddr> {
        sys::getsockname(self.inner)
    }

    /// Returns the socket address of the remote peer of this connection.
    ///
    /// This function directly corresponds to the `getpeername(2)` function.
    pub fn peer_addr(&self) -> io::Result<SockAddr> {
        sys::getpeername(self.inner)
    }

    /// Mark a socket as ready to accept incoming connection requests using
    /// `accept(2)`.
    ///
    /// This function directly corresponds to the `listen(2)` function.
    pub fn listen(&self, backlog: c_int) -> io::Result<()> {
        sys::listen(self.inner, backlog)
    }

    /// Accept a new incoming connection from this listener.
    ///
    /// This function directly corresponds to the `accept(2)` function.
    pub fn accept(&self) -> io::Result<(Socket, SockAddr)> {
        sys::accept(self.inner)
    }

    /// Get the value of the `SO_ERROR` option on this socket.
    ///
    /// This will retrieve the stored error in the underlying socket, clearing
    /// the field in the process. This can be useful for checking errors between
    /// calls.
    pub fn take_error(&self) -> io::Result<Option<io::Error>> {
        self.getsockopt::<c_int>(libc::SOL_SOCKET, libc::SO_ERROR)
            .map(|errno| {
                if errno == 0 {
                    None
                } else {
                    Some(io::Error::from_raw_os_error(errno))
                }
            })
    }

    /// Shuts down the read, write, or both halves of this connection.
    ///
    /// This function will cause all pending and future I/O on the specified
    /// portions to return immediately with an appropriate value.
    pub fn shutdown(&self, how: Shutdown) -> io::Result<()> {
        sys::shutdown(self.inner, how)
    }
}

impl Socket {
    /// Set a socket option.
    ///
    /// This function directly corresponds to the `setsockopt(2)` function. As
    /// different options use different option types the user must define the
    /// correct type `T`!
    pub fn setsockopt<T>(&self, level: c_int, optname: c_int, opt: &T) -> io::Result<()> {
        sys::setsockopt(self.inner, level, optname, opt)
    }

    /// Get a socket option.
    ///
    /// This function directly corresponds to the `getsockopt(2)` function. As
    /// different options have different return types the user must define the
    /// return type `T` correctly!
    ///
    /// For an example usage see [`Socket::take_error`].
    ///
    /// # Notes
    ///
    /// Currently this will panic (in debug mode) if `T` isn't completely
    /// written to, it doesn't support options which partly write to `T`.
    pub fn getsockopt<T>(&self, level: c_int, optname: c_int) -> io::Result<T> {
        sys::getsockopt(self.inner, level, optname)
    }

    /// Manipulate the file descriptor options of the socket.
    ///
    /// This function directly corresponds to the `fcntl(2)` function. As
    /// different command have different options the user must defined the
    /// correct type `T`!
    ///
    /// # Examples
    ///
    /// The following example retrieves and sets the file descriptor flags.
    ///
    /// ```
    /// use std::io;
    /// use socket2::{Domain, Socket, Type};
    ///
    /// # fn main() -> io::Result<()> {
    /// let socket = Socket::new(Domain::IPV4, Type::stream(), None)?;
    ///
    /// // Retrieve the flags, using nothing `()` as argument.
    /// let flags = socket.fcntl(libc::F_GETFD, ())?;
    /// assert!((flags & libc::FD_CLOEXEC) == 0);
    ///
    /// // Now we set the `FD_CLOEXEC` flag.
    /// socket.fcntl(libc::F_SETFD, flags | libc::FD_CLOEXEC)?;
    ///
    /// // Now the flag should be set.
    /// let flags = socket.fcntl(libc::F_GETFD, ())?;
    /// assert!((flags & libc::FD_CLOEXEC) != 0);
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub fn fcntl<T>(&self, cmd: c_int, arg: T) -> io::Result<c_int> {
        sys::fcntl(self.inner, cmd, arg)
    }
}

impl From<TcpStream> for Socket {
    fn from(socket: TcpStream) -> Socket {
        unsafe { Socket::from_raw_fd(socket.into_raw_fd()) }
    }
}

impl Into<TcpStream> for Socket {
    fn into(self) -> TcpStream {
        unsafe { TcpStream::from_raw_fd(self.into_raw_fd()) }
    }
}

impl From<TcpListener> for Socket {
    fn from(socket: TcpListener) -> Socket {
        unsafe { Socket::from_raw_fd(socket.into_raw_fd()) }
    }
}

impl Into<TcpListener> for Socket {
    fn into(self) -> TcpListener {
        unsafe { TcpListener::from_raw_fd(self.into_raw_fd()) }
    }
}

impl From<UdpSocket> for Socket {
    fn from(socket: UdpSocket) -> Socket {
        unsafe { Socket::from_raw_fd(socket.into_raw_fd()) }
    }
}

impl Into<UdpSocket> for Socket {
    fn into(self) -> UdpSocket {
        unsafe { UdpSocket::from_raw_fd(self.into_raw_fd()) }
    }
}

impl fmt::Debug for Socket {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.inner.fmt(f)
    }
}
