// Not all tests use all functions.
#![allow(dead_code)]

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Once;
use std::{env, fs};

/// Bind to any port on localhost.
pub fn any_local_ipv4_addr() -> SocketAddr {
    "127.0.0.1:0".parse().unwrap()
}

/* TODO: needed?
/// Bind to any port on localhost, using a IPv6 address.
pub fn any_local_ipv6_addr() -> SocketAddr {
    "[::1]:0".parse().unwrap()
}
*/

/// Returns a path to a temporary file using `name` as filename.
pub fn temp_file(name: &'static str) -> PathBuf {
    init();
    let mut path = temp_dir();
    path.push(name);
    path
}

pub fn init() {
    static INIT: Once = Once::new();

    INIT.call_once(|| {
        // Remove all temporary files from previous test runs.
        let dir = temp_dir();
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("unable to create temporary directory");
    })
}

/// Returns the temporary directory for test files.
///
/// # Notes
///
/// `init` must be called before this.
fn temp_dir() -> PathBuf {
    let mut path = env::temp_dir();
    path.push("socket_tests");
    path
}
