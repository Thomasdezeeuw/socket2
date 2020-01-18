//! Tests for Unix only API.

#![cfg(unix)]

use std::os::unix::net::{UnixDatagram, UnixListener, UnixStream};

use socket2::{Domain, Socket, Type};

mod util;
use util::temp_file;

#[test]
fn from_std_unix_stream() {
    let path = temp_file("from_std_unix_stream");
    let listener = UnixListener::bind(&path).unwrap();
    let stream = UnixStream::connect(&path).unwrap();
    let socket: Socket = stream.into();
    drop(socket);
    drop(listener);
}

#[test]
fn from_std_unix_listener() {
    let path = temp_file("from_std_unix_listener");
    let listener = UnixListener::bind(&path).unwrap();
    let socket: Socket = listener.into();
    drop(socket);
}

#[test]
fn from_std_unix_socket() {
    let path = temp_file("from_std_unix_socket");
    let datagram = UnixDatagram::bind(&path).unwrap();
    let socket: Socket = datagram.into();
    drop(socket);
}

#[test]
fn into_std_unix_stream() {
    let socket: Socket = Socket::new(Domain::unix(), Type::stream(), None).unwrap();
    let unix_socket: UnixStream = socket.into();
    drop(unix_socket);
}

#[test]
fn into_std_tcp_listener() {
    let socket: Socket = Socket::new(Domain::unix(), Type::stream(), None).unwrap();
    let unix_socket: UnixListener = socket.into();
    drop(unix_socket);
}

#[test]
fn into_std_udp_socket() {
    let socket: Socket = Socket::new(Domain::unix(), Type::dgram(), None).unwrap();
    let unix_socket: UnixDatagram = socket.into();
    drop(unix_socket);
}

// TODO: test accept4.
// TODO: test pair.
