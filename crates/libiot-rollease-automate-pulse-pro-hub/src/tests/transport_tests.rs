//! Tests for [`crate::transport::Transport`] using `tokio::io::duplex`
//! pairs as in-memory stand-ins for a real TCP stream.
//!
//! Every test constructs a duplex pair, hands one half to
//! `Transport::new`, and uses the other half as a scripted "fake hub"
//! that can verify the bytes the transport writes and feed back
//! canned response bytes. No real sockets are opened anywhere in
//! this file.

use std::time::Duration;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;
use tokio::io::duplex;

use crate::codec::IncomingFrame;
use crate::error::Error;
use crate::transport::Transport;

/// Verifies that `Transport::write_frames` writes the exact caller-
/// provided bytes to the underlying stream and flushes them
/// immediately so a downstream reader sees them right away.
///
/// Written by Claude Code, reviewed by a human.
#[tokio::test]
async fn write_frames_delivers_exact_bytes_to_the_stream() {
    let (client_side, mut hub_side) = duplex(/* capacity = */ 1024);
    let mut transport = Transport::new(client_side);

    // Issue a write on the client side.
    transport.write_frames(b"!4JKo;").await.unwrap();

    // Read the same bytes from the hub side.
    let mut read_buf = [0u8; 6];
    hub_side.read_exact(&mut read_buf).await.unwrap();
    assert_eq!(&read_buf, b"!4JKo;");
}

/// Verifies that `Transport::read_until` parses a canned response from
/// the hub side into the expected [`IncomingFrame`] and returns early
/// as soon as the `is_done` predicate is satisfied.
///
/// Written by Claude Code, reviewed by a human.
#[tokio::test]
async fn read_until_returns_when_predicate_is_satisfied() {
    let (client_side, mut hub_side) = duplex(1024);
    let mut transport = Transport::new(client_side);

    // Hub writes a hub-name reply.
    hub_side
        .write_all(b"!000NAME6217 Shade Hub;")
        .await
        .unwrap();
    hub_side.flush().await.unwrap();

    let frames = transport
        .read_until(
            |frames| {
                frames
                    .iter()
                    .any(|f| matches!(f, IncomingFrame::HubName(_)))
            },
            Duration::from_millis(500),
        )
        .await
        .unwrap();

    assert_eq!(frames.len(), 1);
    assert!(matches!(&frames[0], IncomingFrame::HubName(n) if n == "6217 Shade Hub"));
}

/// Verifies that `Transport::read_until` correctly reassembles a frame
/// that the hub side writes in two pieces — the parser/transport stack
/// must handle partial-read accumulation for the real-world firehose
/// pattern described in §2.9 quirk #1 of `PULSE_PRO_LOCAL_API.md`.
///
/// Written by Claude Code, reviewed by a human.
#[tokio::test]
async fn read_until_reassembles_frame_split_across_two_writes() {
    let (client_side, mut hub_side) = duplex(1024);
    let mut transport = Transport::new(client_side);

    // First half arrives, doesn't contain a complete frame.
    hub_side.write_all(b"!000NAME6217 Shad").await.unwrap();
    hub_side.flush().await.unwrap();

    // Second half arrives shortly after (in a separate task so the
    // transport's read loop isn't starved while we run synchronous
    // setup here).
    let delayed_finish = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(20)).await;
        hub_side.write_all(b"e Hub;").await.unwrap();
        hub_side.flush().await.unwrap();
    });

    let frames = transport
        .read_until(|frames| !frames.is_empty(), Duration::from_millis(500))
        .await
        .unwrap();

    delayed_finish.await.unwrap();

    assert!(matches!(&frames[0], IncomingFrame::HubName(n) if n == "6217 Shade Hub"));
}

/// Verifies that `Transport::read_until` returns `Error::Timeout` when
/// the hub side never writes anything within the caller-supplied
/// timeout.
///
/// Written by Claude Code, reviewed by a human.
#[tokio::test]
async fn read_until_returns_timeout_when_nothing_arrives() {
    let (client_side, _hub_side) = duplex(1024);
    let mut transport = Transport::new(client_side);

    let result = transport
        .read_until(|_| false, Duration::from_millis(50))
        .await;

    match result {
        Err(Error::Timeout { ms }) => assert!(ms >= 50),
        other => panic!("expected Timeout, got {other:?}"),
    }
}

/// Verifies that `Transport::read_for` collects every frame that
/// arrives within the window, regardless of count, and stops when the
/// window elapses. This is the path used for broadcast queries like
/// `!000r?;` where the client doesn't know ahead of time how many
/// motors will respond.
///
/// Written by Claude Code, reviewed by a human.
#[tokio::test]
async fn read_for_collects_all_frames_within_the_window() {
    let (client_side, mut hub_side) = duplex(4096);
    let mut transport = Transport::new(client_side);

    // Hub writes the real `6217 Shade Hub` position-query burst.
    hub_side
        .write_all(b"!4JKr000b000,R58;!MWXr100b180,R4C;!3YCr000b000,R4C;")
        .await
        .unwrap();
    hub_side.flush().await.unwrap();

    let frames = transport
        .read_for(Duration::from_millis(150))
        .await
        .unwrap();

    assert_eq!(
        frames.len(),
        3,
        "expected 3 position frames, got {}: {frames:?}",
        frames.len(),
    );
}
