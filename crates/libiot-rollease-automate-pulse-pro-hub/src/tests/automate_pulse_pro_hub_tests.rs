//! Integration-style tests for [`crate::AutomatePulseProHub`] using an
//! in-process fake hub. Each test binds a `tokio::net::TcpListener` to
//! `127.0.0.1:0`, spawns a scripted "fake hub" task on the listener's
//! accepted connection, and exercises the real client against it.
//!
//! No external network access is involved — `127.0.0.1:0` always
//! succeeds and the kernel picks a free ephemeral port, so these
//! tests are hermetic and can run in parallel.

use std::time::Duration;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;

use crate::automate_pulse_pro_hub::AutomatePulseProHub;
use crate::error::Error;
use crate::error::HubErrorCode;
use crate::motor_address::MotorAddress;
use crate::motor_type::MotorType;

/// Bind a loopback `TcpListener`, return (listener, "ip:port" string)
/// for handing to `AutomatePulseProHub::connect`.
async fn bind_listener() -> (TcpListener, String) {
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("can bind loopback port 0");
    let local = listener.local_addr().expect("listener has a local addr");
    (listener, format!("{local}"))
}

/// Read exactly `expected.len()` bytes from the socket and assert they
/// match `expected`. Used inside fake-hub tasks to verify the client
/// wrote the right command.
async fn expect_bytes(socket: &mut TcpStream, expected: &[u8]) {
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

// -- fire-and-forget commands ---------------------------------------------

/// Verifies that `AutomatePulseProHub::close` writes the exact
/// `!<addr>c;` bytes to the hub, and that `connect` succeeds against
/// a local loopback TCP listener acting as a fake hub.
///
/// Written by Claude Code, reviewed by a human.
#[tokio::test]
async fn close_writes_close_command_bytes_to_the_hub() {
    let (listener, addr_str) = bind_listener().await;

    let fake_hub = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        expect_bytes(&mut socket, b"!4JKc;").await;
        // Keep the socket open until the test drops the client.
        let _ = socket.shutdown().await;
    });

    let hub = AutomatePulseProHub::connect(&addr_str).await.unwrap();
    let motor = MotorAddress::new("4JK").unwrap();
    hub.close(&motor).await.unwrap();

    fake_hub.await.unwrap();
}

/// Verifies that `AutomatePulseProHub::open_all` writes the broadcast
/// open command `!000o;`.
///
/// Written by Claude Code, reviewed by a human.
#[tokio::test]
async fn open_all_writes_broadcast_open_bytes() {
    let (listener, addr_str) = bind_listener().await;

    let fake_hub = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        expect_bytes(&mut socket, b"!000o;").await;
        let _ = socket.shutdown().await;
    });

    let hub = AutomatePulseProHub::connect(&addr_str).await.unwrap();
    hub.open_all().await.unwrap();

    fake_hub.await.unwrap();
}

/// Verifies that `AutomatePulseProHub::set_position` writes the exact
/// `!<addr>m<NNN>;` bytes with 3-digit padding (the documented
/// field-verified quirk from §2.5 of `PULSE_PRO_LOCAL_API.md`).
///
/// Written by Claude Code, reviewed by a human.
#[tokio::test]
async fn set_position_writes_three_digit_padded_move_command() {
    let (listener, addr_str) = bind_listener().await;

    let fake_hub = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        expect_bytes(&mut socket, b"!MWXm050;").await;
        let _ = socket.shutdown().await;
    });

    let hub = AutomatePulseProHub::connect(&addr_str).await.unwrap();
    let motor = MotorAddress::new("MWX").unwrap();
    hub.set_position(&motor, /* percent = */ 50).await.unwrap();

    fake_hub.await.unwrap();
}

// -- query responses ------------------------------------------------------

/// Verifies that `AutomatePulseProHub::hub_name` writes the
/// `!000NAME?;` query and returns the string from the matching reply.
///
/// Written by Claude Code, reviewed by a human.
#[tokio::test]
async fn hub_name_query_returns_name_from_canned_reply() {
    let (listener, addr_str) = bind_listener().await;

    let fake_hub = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        expect_bytes(&mut socket, b"!000NAME?;").await;
        socket.write_all(b"!000NAME6217 Shade Hub;").await.unwrap();
        socket.flush().await.unwrap();
        let _ = socket.shutdown().await;
    });

    let hub = AutomatePulseProHub::connect(&addr_str).await.unwrap();
    let name = hub.hub_name().await.unwrap();
    assert_eq!(name, "6217 Shade Hub");

    fake_hub.await.unwrap();
}

/// Verifies that `AutomatePulseProHub::motor_position` writes the
/// `!<addr>r?;` query, parses the 3-digit / 3-digit / 2-hex reply
/// correctly, and returns the parsed fields.
///
/// Written by Claude Code, reviewed by a human.
#[tokio::test]
async fn motor_position_query_parses_real_reply_fields() {
    let (listener, addr_str) = bind_listener().await;

    let fake_hub = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        expect_bytes(&mut socket, b"!MWXr?;").await;
        socket.write_all(b"!MWXr100b180,R4C;").await.unwrap();
        socket.flush().await.unwrap();
        let _ = socket.shutdown().await;
    });

    let hub = AutomatePulseProHub::connect(&addr_str).await.unwrap();
    let motor = MotorAddress::new("MWX").unwrap();
    let position = hub.motor_position(&motor).await.unwrap();
    assert_eq!(position.closed_percent, 100);
    assert_eq!(position.tilt_percent, 180);
    assert_eq!(position.signal, 0x4C);

    fake_hub.await.unwrap();
}

/// Verifies that when the hub replies with a typed error frame
/// (`!<addr>E<xx>;`), the client surfaces it as
/// `Error::HubError { address, code }` — specifically the
/// `MotorOffline` variant for `nl`, which is the most common error in
/// day-to-day use.
///
/// Written by Claude Code, reviewed by a human.
#[tokio::test]
async fn query_surfaces_hub_error_frame_as_typed_error() {
    let (listener, addr_str) = bind_listener().await;

    let fake_hub = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        expect_bytes(&mut socket, b"!4JKr?;").await;
        socket.write_all(b"!4JKEnl;").await.unwrap();
        socket.flush().await.unwrap();
        let _ = socket.shutdown().await;
    });

    let hub = AutomatePulseProHub::connect(&addr_str).await.unwrap();
    let motor = MotorAddress::new("4JK").unwrap();
    let result = hub.motor_position(&motor).await;

    match result {
        Err(Error::HubError { address, code }) => {
            assert_eq!(address, motor);
            assert_eq!(code, HubErrorCode::MotorOffline);
        },
        other => panic!("expected HubError, got {other:?}"),
    }

    fake_hub.await.unwrap();
}

// -- info (the highest-value integration test) -----------------------

/// Verifies that `AutomatePulseProHub::info` performs the
/// documented two-batch query pattern (hub metadata + position enum,
/// then per-motor name queries) and correctly stitches the results
/// into a `HubInfo`. Uses the real captured bytes from §6 of
/// `PULSE_PRO_LOCAL_API.md` as the hub replies.
///
/// Written by Claude Code, reviewed by a human.
#[tokio::test]
async fn snapshot_matches_real_hub_capture_from_spec() {
    let (listener, addr_str) = bind_listener().await;

    let fake_hub = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();

        // Batch 1: hub name, serial, motor enum, position enum.
        expect_bytes(&mut socket, b"!000NAME?;!000SN?;!000v?;!000r?;").await;
        socket
            .write_all(
                b"!000NAME6217 Shade Hub;!000SN2016197;\
                  !BR1vB10;!4JKvD22;!MWXvD22;!3YCvD22;\
                  !4JKr000b000,R58;!MWXr100b180,R4C;!3YCr000b000,R4C;",
            )
            .await
            .unwrap();
        socket.flush().await.unwrap();

        // Batch 2: per-motor friendly-name queries for the three
        // discovered motors (in enumeration order).
        expect_bytes(&mut socket, b"!4JKNAME?;!MWXNAME?;!3YCNAME?;").await;
        socket
            .write_all(b"!4JKNAMEJohn House;!MWXNAMEDining Room;!3YCNAMEKitchen;")
            .await
            .unwrap();
        socket.flush().await.unwrap();

        let _ = socket.shutdown().await;
    });

    let hub = AutomatePulseProHub::connect(&addr_str).await.unwrap();
    let info = hub.info().await.unwrap();

    assert_eq!(info.hub_name, "6217 Shade Hub");
    assert_eq!(info.hub_serial, "2016197");
    assert_eq!(info.motors.len(), 3, "hub gateway should be filtered");

    // Motor 0 — 4JK "John House", 0% closed, DC motor.
    let m0 = &info.motors[0];
    assert_eq!(m0.address, MotorAddress::new("4JK").unwrap());
    assert_eq!(m0.name.as_deref(), Some("John House"));
    assert_eq!(m0.version.motor_type, MotorType::Dc);
    let p0 = m0.position.expect("4JK should have a position");
    assert_eq!(p0.closed_percent, 0);

    // Motor 1 — MWX "Dining Room", 100% closed, tilt 180.
    let m1 = &info.motors[1];
    assert_eq!(m1.address, MotorAddress::new("MWX").unwrap());
    assert_eq!(m1.name.as_deref(), Some("Dining Room"));
    let p1 = m1.position.expect("MWX should have a position");
    assert_eq!(p1.closed_percent, 100);
    assert_eq!(p1.tilt_percent, 180);

    // Motor 2 — 3YC "Kitchen", 0% closed.
    let m2 = &info.motors[2];
    assert_eq!(m2.address, MotorAddress::new("3YC").unwrap());
    assert_eq!(m2.name.as_deref(), Some("Kitchen"));

    // Read timeout is generous (3s + 3s = 6s) so the test should complete
    // well under that window. Bump this if it starts flaking.
    tokio::time::timeout(Duration::from_secs(10), fake_hub)
        .await
        .unwrap()
        .unwrap();
}

/// Verifies that `AutomatePulseProHub::hub_name` times out cleanly
/// when the hub accepts the connection but never writes a response.
///
/// Written by Claude Code, reviewed by a human.
#[tokio::test]
async fn hub_name_times_out_when_hub_never_replies() {
    let (listener, addr_str) = bind_listener().await;

    let fake_hub = tokio::spawn(async move {
        let (mut socket, _) = listener.accept().await.unwrap();
        // Drain the incoming query but deliberately never reply.
        let mut sink = [0u8; 32];
        let _ = socket.read(&mut sink).await;
        // Hold the connection open long enough for the client-side
        // timeout to fire.
        tokio::time::sleep(Duration::from_millis(400)).await;
    });

    let hub = AutomatePulseProHub::connect(&addr_str).await.unwrap();
    let result = hub.hub_name().await;
    match result {
        Err(Error::Timeout { .. }) => {},
        other => panic!("expected Timeout, got {other:?}"),
    }

    let _ = fake_hub.await;
}
