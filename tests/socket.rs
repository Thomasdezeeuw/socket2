use std::net::{TcpListener, TcpStream, UdpSocket};

use socket2::{Domain, Socket, Type};

mod util;
use util::any_local_ipv4_addr;

#[test]
fn from_std_tcp_stream() {
    let listener = TcpListener::bind(any_local_ipv4_addr()).unwrap();
    let tcp_socket = TcpStream::connect(listener.local_addr().unwrap()).unwrap();
    let socket: Socket = tcp_socket.into();
    drop(socket);
}

#[test]
fn from_std_tcp_listener() {
    let tcp_socket = TcpListener::bind(any_local_ipv4_addr()).unwrap();
    let socket: Socket = tcp_socket.into();
    drop(socket);
}

#[test]
fn from_std_udp_socket() {
    let udp_socket = UdpSocket::bind(any_local_ipv4_addr()).unwrap();
    let socket: Socket = udp_socket.into();
    drop(socket);
}

#[test]
fn into_std_tcp_stream() {
    let socket: Socket = Socket::new(Domain::ipv4(), Type::stream(), None).unwrap();
    let tcp_socket: TcpStream = socket.into();
    drop(tcp_socket);
}

#[test]
fn into_std_tcp_listener() {
    let socket: Socket = Socket::new(Domain::ipv4(), Type::stream(), None).unwrap();
    let tcp_socket: TcpListener = socket.into();
    drop(tcp_socket);
}

#[test]
fn into_std_udp_socket() {
    let socket: Socket = Socket::new(Domain::ipv4(), Type::dgram(), None).unwrap();
    let udp_socket: UdpSocket = socket.into();
    drop(udp_socket);
}

#[test]
fn socket_connect_tcp() {
    let listener = TcpListener::bind(any_local_ipv4_addr()).unwrap();
    let addr = listener.local_addr().unwrap();

    let socket: TcpStream = Socket::new(Domain::ipv4(), Type::stream(), None)
        .and_then(|socket| socket.connect(&addr.into()).map(|()| socket.into()))
        .unwrap();
    assert_eq!(socket.peer_addr().unwrap(), addr);

    let (stream, peer_addr) = listener.accept().unwrap();
    let socket_local_addr = socket.local_addr().unwrap();
    assert_eq!(peer_addr, socket_local_addr);
    assert_eq!(stream.peer_addr().unwrap(), socket_local_addr);
}

#[test]
fn socket_bind_tcp() {
    let socket: TcpListener = Socket::new(Domain::ipv4(), Type::stream(), None)
        .and_then(|socket| {
            socket
                .bind(&any_local_ipv4_addr().into())
                .map(|()| socket.into())
        })
        .unwrap();

    assert!(socket.local_addr().unwrap().ip().is_loopback())
}

#[test]
fn socket_listen_tcp() {
    let socket: TcpListener = Socket::new(Domain::ipv4(), Type::stream(), None)
        .and_then(|socket| {
            socket.bind(&any_local_ipv4_addr().into())?;
            socket.listen(1024)?;
            Ok(socket.into())
        })
        .unwrap();
    let addr = socket.local_addr().unwrap();

    let stream = TcpStream::connect(addr).unwrap();
    let stream_addr = stream.local_addr().unwrap();

    let (accepted_stream, peer_addr) = socket.accept().unwrap();
    assert_eq!(peer_addr, stream_addr);
    assert_eq!(accepted_stream.peer_addr().unwrap(), stream_addr);
}

// Also tests `local_addr` and `peer_addr`.
#[test]
fn socket_accept_tcp() {
    let socket: Socket = Socket::new(Domain::ipv4(), Type::stream(), None)
        .and_then(|socket| {
            socket.bind(&any_local_ipv4_addr().into())?;
            socket.listen(1024)?;
            Ok(socket.into())
        })
        .unwrap();
    let addr = socket.local_addr().unwrap();
    let addr = addr.as_std().unwrap();

    let stream = TcpStream::connect(addr).unwrap();
    let stream_addr = stream.local_addr().unwrap();

    let (accepted_socket, peer_addr) = socket.accept().unwrap();
    let peer_addr = peer_addr.as_std().unwrap();
    assert_eq!(peer_addr, stream_addr);
    assert_eq!(
        accepted_socket.peer_addr().unwrap().as_std().unwrap(),
        stream_addr
    );
}
