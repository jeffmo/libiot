//! Generic transport layer over `tokio::io::AsyncRead + AsyncWrite`.
//!
//! This module is deliberately generic over any type implementing
//! [`tokio::io::AsyncRead`] + [`tokio::io::AsyncWrite`] + [`Unpin`],
//! not specifically [`tokio::net::TcpStream`]. That way the unit tests
//! can substitute `tokio::io::duplex()` pairs for a real socket and
//! exercise every read/write path in-process — see
//! `crate::tests::transport_tests` for the test suite.

// `Transport<S>` has no non-test consumer at this commit — the public
// `AutomatePulseProHub` client that wraps it lands in the next commit.
// This module-level allow is removed in that commit, at which point
// every method here has a real caller in the non-test build graph.
#![allow(dead_code)]

use std::time::Duration;
use std::time::Instant;

use tokio::io::AsyncRead;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWrite;
use tokio::io::AsyncWriteExt;

use crate::codec::IncomingFrame;
use crate::codec::parse_frames;
use crate::error::Error;
use crate::error::Result;

/// Size of the read-side temporary buffer used inside [`Transport`].
/// Each call to [`tokio::io::AsyncReadExt::read`] can fill up to this
/// many bytes; completed frames are drained out of a larger internal
/// accumulator buffer after each read.
const READ_CHUNK_BYTES: usize = 512;

/// Async transport wrapping any `AsyncRead + AsyncWrite` stream.
///
/// The transport owns a small read accumulator so it can handle the
/// firehose pattern documented in §2.9 quirk #1 of the in-crate
/// `PULSE_PRO_LOCAL_API.md`: partial frames split across multiple
/// `read`s, multiple frames arriving in one `read`, and unsolicited
/// frames interleaved with responses to a client query.
pub(crate) struct Transport<S: AsyncRead + AsyncWrite + Unpin> {
    stream: S,
    accum: Vec<u8>,
}

impl<S: AsyncRead + AsyncWrite + Unpin> Transport<S> {
    /// Wrap an existing stream.
    pub(crate) fn new(stream: S) -> Self {
        Self {
            stream,
            accum: Vec::with_capacity(READ_CHUNK_BYTES),
        }
    }

    /// Write `bytes` to the stream in one write and flush.
    ///
    /// The Pulse Pro hub's ASCII protocol allows (and prefers) batching
    /// multiple frames into a single TCP write. Callers should
    /// pre-concatenate their batch and hand the whole buffer to this
    /// method in one shot.
    pub(crate) async fn write_frames(&mut self, bytes: &[u8]) -> Result<()> {
        self.stream.write_all(bytes).await?;
        self.stream.flush().await?;
        Ok(())
    }

    /// Read frames from the stream until `is_done` returns `true` on
    /// the accumulated list of frames so far, or until `timeout`
    /// elapses — whichever comes first.
    ///
    /// Every frame read off the stream (whether "expected" or not) is
    /// included in the returned vector. The caller is responsible for
    /// filtering out frames it doesn't care about — the transport
    /// itself has no notion of request/response correlation.
    ///
    /// Returns [`Error::Timeout`] if the timeout elapses before
    /// `is_done` returns `true`.
    pub(crate) async fn read_until<F>(
        &mut self,
        mut is_done: F,
        timeout: Duration,
    ) -> Result<Vec<IncomingFrame>>
    where
        F: FnMut(&[IncomingFrame]) -> bool,
    {
        let deadline = Instant::now() + timeout;
        let mut collected: Vec<IncomingFrame> = Vec::new();

        // First, drain any frames already in the accumulator from a prior
        // read call. This means "is_done" may be satisfied before we even
        // touch the socket.
        if !self.accum.is_empty() {
            let newly_parsed = parse_frames(&mut self.accum)?;
            collected.extend(newly_parsed);
            if is_done(&collected) {
                return Ok(collected);
            }
        }

        let mut chunk = [0u8; READ_CHUNK_BYTES];
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(Error::Timeout {
                    ms: millis_of(timeout),
                });
            }

            let read_fut = self.stream.read(&mut chunk);
            let read_result = tokio::time::timeout(remaining, read_fut).await;

            let n = match read_result {
                Ok(Ok(n)) => n,
                Ok(Err(io_err)) => return Err(Error::Io(io_err)),
                Err(_elapsed) => {
                    return Err(Error::Timeout {
                        ms: millis_of(timeout),
                    });
                },
            };

            if n == 0 {
                // Stream closed with nothing left in-flight; treat as a
                // timeout if the caller still wanted more frames.
                return Err(Error::Timeout {
                    ms: millis_of(timeout),
                });
            }

            self.accum.extend_from_slice(&chunk[..n]);
            let newly_parsed = parse_frames(&mut self.accum)?;
            collected.extend(newly_parsed);

            if is_done(&collected) {
                return Ok(collected);
            }
        }
    }

    /// Read frames for `duration`, regardless of what arrives. Used for
    /// broadcast queries where the number of responses depends on how
    /// many motors are online — no early termination is possible.
    ///
    /// A broadcast query's responses may dribble in over several tens
    /// of milliseconds because the hub dispatches them one by one onto
    /// its 433 MHz radio. `read_for` keeps reading until the duration
    /// fully elapses and returns every frame that arrived in that
    /// window.
    pub(crate) async fn read_for(&mut self, duration: Duration) -> Result<Vec<IncomingFrame>> {
        let mut collected: Vec<IncomingFrame> = Vec::new();

        if !self.accum.is_empty() {
            let newly_parsed = parse_frames(&mut self.accum)?;
            collected.extend(newly_parsed);
        }

        let deadline = Instant::now() + duration;
        let mut chunk = [0u8; READ_CHUNK_BYTES];
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }
            let read_fut = self.stream.read(&mut chunk);
            match tokio::time::timeout(remaining, read_fut).await {
                Ok(Ok(0)) => break, // stream closed
                Ok(Ok(n)) => {
                    self.accum.extend_from_slice(&chunk[..n]);
                    let newly_parsed = parse_frames(&mut self.accum)?;
                    collected.extend(newly_parsed);
                },
                Ok(Err(io_err)) => return Err(Error::Io(io_err)),
                Err(_elapsed) => break, // timeout: we're done, return what we got
            }
        }

        Ok(collected)
    }
}

/// Convert a [`Duration`] to a `u64` millisecond count, saturating on
/// overflow. Used exclusively to populate [`Error::Timeout`].
fn millis_of(d: Duration) -> u64 {
    u64::try_from(d.as_millis()).unwrap_or(u64::MAX)
}
