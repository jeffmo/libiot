//! Shared test helpers — fake-hub TCP listener for e2e tests.
//
// Not every integration test file uses every helper, so individual
// helpers may appear dead in one compilation unit but used in another.
#![allow(dead_code)]

use std::net::SocketAddr;

use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

/// Bind a loopback `TcpListener` on an ephemeral port and return both
/// the listener and the `"HOST:PORT"` string suitable for passing to
/// `--hub`.
pub async fn bind_listener() -> (TcpListener, String) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("can bind loopback ephemeral port");
    let local: SocketAddr = listener.local_addr().expect("listener has a local addr");
    (listener, format!("{local}"))
}

/// Read exactly `expected.len()` bytes from the socket and assert they
/// match `expected`.
pub async fn expect_bytes(socket: &mut TcpStream, expected: &[u8]) {
    let mut buf = vec![0u8; expected.len()];
    socket
        .read_exact(&mut buf)
        .await
        .unwrap_or_else(|err| panic!("fake hub could not read {} bytes: {err}", expected.len()));
    assert_eq!(
        &buf,
        expected,
        "unexpected bytes on wire: got {:?}, expected {:?}",
        String::from_utf8_lossy(&buf),
        String::from_utf8_lossy(expected),
    );
}

/// Accept one connection on `listener`, run `handler` with the socket,
/// then shut down cleanly.
pub async fn serve_one<F, Fut>(listener: TcpListener, handler: F)
where
    F: FnOnce(TcpStream) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send,
{
    let (socket, _) = listener.accept().await.unwrap();
    handler(socket).await;
}
