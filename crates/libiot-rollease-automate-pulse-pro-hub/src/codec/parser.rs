//! Streaming frame parser.
//!
//! The parser consumes bytes from an owned `&mut Vec<u8>` so it can
//! handle the firehose pattern described in §2.9 quirk #1 of the
//! in-crate `PULSE_PRO_LOCAL_API.md` — the hub may push unsolicited
//! frames and may split a single logical response across multiple
//! TCP reads. The parser drains all complete frames (delimited by
//! `;`) from the front of the buffer and leaves any partial trailing
//! frame in place for the next call.

use crate::codec::incoming_frame::IncomingFrame;
use crate::error::Error;
use crate::error::HubErrorCode;
use crate::error::Result;
use crate::motor_address::MotorAddress;
use crate::motor_position::MotorPosition;
use crate::motor_type::MotorType;
use crate::motor_voltage::MotorVoltage;

/// Drain every complete frame from the front of `buf`, returning them in
/// the order the hub sent them. Partial trailing frames are left in the
/// buffer for a later call.
///
/// This function never panics on arbitrary input. Malformed frame
/// structure surfaces as [`Error::Malformed`]; unknown hub error codes
/// surface as [`HubErrorCode::Unknown`], not a parse error. The parser
/// tolerates stray CR/LF bytes inside a frame by stripping them before
/// classification.
pub(crate) fn parse_frames(buf: &mut Vec<u8>) -> Result<Vec<IncomingFrame>> {
    let mut out = Vec::new();

    // Find frame boundaries (`;`) from the front of the buffer.
    let mut consumed = 0;
    while let Some(end_offset) = memchr_semicolon(&buf[consumed..]) {
        // `end_offset` is relative to `consumed`, and points at the `;` byte.
        let frame_start = consumed;
        let frame_end = consumed + end_offset; // exclusive of `;`
        let frame_bytes = &buf[frame_start..frame_end];

        // Strip stray CR/LF from the frame contents.
        let cleaned: Vec<u8> = frame_bytes
            .iter()
            .copied()
            .filter(|&b| b != b'\r' && b != b'\n')
            .collect();

        if !cleaned.is_empty() {
            let frame = parse_single_frame(&cleaned)?;
            out.push(frame);
        }

        // Advance past the `;`.
        consumed = frame_end + 1;
    }

    // Remove the consumed bytes from the front of the buffer. Any trailing
    // partial frame stays put.
    buf.drain(..consumed);

    Ok(out)
}

/// Find the first `;` byte in `slice` (dependency-free memchr).
fn memchr_semicolon(slice: &[u8]) -> Option<usize> {
    slice.iter().position(|&b| b == b';')
}

/// Parse a single frame's bytes (already stripped of the surrounding
/// `!` and `;` delimiters, and any CR/LF noise).
fn parse_single_frame(bytes: &[u8]) -> Result<IncomingFrame> {
    if bytes.is_empty() {
        return Err(malformed("empty frame", bytes));
    }
    if bytes[0] != b'!' {
        return Err(malformed("frame does not start with '!'", bytes));
    }
    if bytes.len() < 4 {
        return Err(malformed("frame too short for address + command", bytes));
    }

    // After the leading '!': 3 address bytes, then a command byte and
    // variable payload.
    let addr_bytes = &bytes[1..4];
    let after_addr = &bytes[4..];

    let addr_str = std::str::from_utf8(addr_bytes)
        .map_err(|_| malformed("address bytes are not valid UTF-8", bytes))?;

    // Hub-level frames for addresses `000` (broadcast / hub-wide).
    // Hub name and serial live under the `000` address with a non-single
    // command byte, so they need a special case before the per-motor path.
    if addr_bytes == b"000" {
        if let Some(rest) = after_addr.strip_prefix(b"NAME") {
            return Ok(IncomingFrame::HubName(string_from_utf8(rest, bytes)?));
        }
        if let Some(rest) = after_addr.strip_prefix(b"SN") {
            return Ok(IncomingFrame::HubSerial(string_from_utf8(rest, bytes)?));
        }
        // Fall through — some `000`-addressed replies (e.g. broadcast
        // position frames are returned as individual motor replies, not
        // `000`-addressed ones, so we don't need to special-case them).
    }

    // Motor-level frames. The address validates as a full MotorAddress
    // since the hub never emits non-alphanumeric addresses in replies.
    let motor_addr = MotorAddress::new(addr_str).map_err(|_| {
        malformed(
            &format!("address {addr_str:?} is not 3 chars of [0-9A-Za-z]"),
            bytes,
        )
    })?;

    // Single-byte command dispatch.
    let (cmd_byte, payload) = (after_addr[0], &after_addr[1..]);

    match cmd_byte {
        b'N' => {
            // `NAME<string>` motor friendly name.
            let rest = after_addr
                .strip_prefix(b"NAME")
                .ok_or_else(|| malformed("expected NAME prefix", bytes))?;
            Ok(IncomingFrame::MotorName {
                addr: motor_addr,
                name: string_from_utf8(rest, bytes)?,
            })
        },
        b'v' => parse_version_rec(motor_addr, payload, bytes),
        b'r' => parse_position_rec(motor_addr, payload, bytes),
        b'p' => parse_voltage_rec(motor_addr, after_addr, bytes),
        b'E' => parse_error_rec(motor_addr, payload, bytes),
        b'A' if payload.is_empty() => Ok(IncomingFrame::AddressAck { addr: motor_addr }),
        _ => Err(malformed(
            &format!("unknown command byte {cmd_byte:?}"),
            bytes,
        )),
    }
}

/// Parse the payload of a `!<addr>v<type><ver>;` frame (the `v` has
/// already been consumed).
fn parse_version_rec(
    addr: MotorAddress,
    payload: &[u8],
    full_frame: &[u8],
) -> Result<IncomingFrame> {
    if payload.is_empty() {
        return Err(malformed("empty version payload", full_frame));
    }
    let motor_type = MotorType::from_wire_byte(payload[0]).ok_or_else(|| {
        malformed(
            &format!("unknown motor type byte {:?}", payload[0]),
            full_frame,
        )
    })?;
    let version = string_from_utf8(&payload[1..], full_frame)?;
    Ok(IncomingFrame::MotorVersionRec {
        addr,
        motor_type,
        version,
    })
}

/// Parse the payload of a `!<addr>r<closed3>b<tilt3>,R<sig2>;` frame (the
/// `r` has already been consumed).
fn parse_position_rec(
    addr: MotorAddress,
    payload: &[u8],
    full_frame: &[u8],
) -> Result<IncomingFrame> {
    // Expect: `<closed3>b<tilt3>,R<sig2>`
    if payload.len() < 3 {
        return Err(malformed(
            "position payload too short for closed3",
            full_frame,
        ));
    }
    let closed_digits = &payload[0..3];
    let closed = parse_three_digit(closed_digits, full_frame)?;

    let rest = &payload[3..];
    let rest = rest
        .strip_prefix(b"b")
        .ok_or_else(|| malformed("expected 'b' after closed3", full_frame))?;

    // The tilt field is documented as 3 digits but is sometimes longer —
    // a real hub has been observed emitting `180` (valid 3-digit value)
    // and, in older firmware, longer values. Parse until we hit `,`.
    let comma_idx = rest
        .iter()
        .position(|&b| b == b',')
        .ok_or_else(|| malformed("expected ',' after tilt", full_frame))?;
    let tilt_str = std::str::from_utf8(&rest[..comma_idx])
        .map_err(|_| malformed("tilt bytes are not valid UTF-8", full_frame))?;
    let tilt_percent: u16 = tilt_str.parse().map_err(|_| {
        malformed(
            &format!("tilt {tilt_str:?} is not a decimal integer"),
            full_frame,
        )
    })?;

    // After the `,` we expect `R<sig2>`.
    let sig_section = &rest[comma_idx + 1..];
    let sig_bytes = sig_section
        .strip_prefix(b"R")
        .ok_or_else(|| malformed("expected 'R' after ','", full_frame))?;
    if sig_bytes.len() != 2 {
        return Err(malformed(
            &format!("signal byte should be 2 hex chars, got {}", sig_bytes.len()),
            full_frame,
        ));
    }
    let sig_str = std::str::from_utf8(sig_bytes)
        .map_err(|_| malformed("signal bytes are not valid UTF-8", full_frame))?;
    let signal = u8::from_str_radix(sig_str, 16).map_err(|_| {
        malformed(
            &format!("signal {sig_str:?} is not 2 hex chars"),
            full_frame,
        )
    })?;

    Ok(IncomingFrame::MotorPositionRec {
        addr,
        position: MotorPosition {
            closed_percent: closed,
            tilt_percent,
            signal,
        },
    })
}

/// Parse a voltage frame `!<addr>pVc<NNNNN>,R<sig2>;`. The argument here
/// is the full after-address payload (starting at `p`), because the `p`
/// command byte is the prefix of the longer `pVc` marker.
fn parse_voltage_rec(
    addr: MotorAddress,
    after_addr: &[u8],
    full_frame: &[u8],
) -> Result<IncomingFrame> {
    let rest = after_addr
        .strip_prefix(b"pVc")
        .ok_or_else(|| malformed("expected 'pVc' after address for voltage reply", full_frame))?;

    // Find the `,R<sig2>` trailer.
    let comma_idx = rest
        .iter()
        .position(|&b| b == b',')
        .ok_or_else(|| malformed("expected ',' after voltage digits", full_frame))?;
    let digits = &rest[..comma_idx];
    let digits_str = std::str::from_utf8(digits)
        .map_err(|_| malformed("voltage digits are not valid UTF-8", full_frame))?;
    let centivolts: u32 = digits_str.parse().map_err(|_| {
        malformed(
            &format!("voltage {digits_str:?} not a decimal integer"),
            full_frame,
        )
    })?;

    let sig_section = &rest[comma_idx + 1..];
    let sig_bytes = sig_section
        .strip_prefix(b"R")
        .ok_or_else(|| malformed("expected 'R' after voltage digits", full_frame))?;
    if sig_bytes.len() != 2 {
        return Err(malformed(
            &format!("signal byte should be 2 hex chars, got {}", sig_bytes.len()),
            full_frame,
        ));
    }
    let sig_str = std::str::from_utf8(sig_bytes)
        .map_err(|_| malformed("signal bytes are not valid UTF-8", full_frame))?;
    let signal = u8::from_str_radix(sig_str, 16).map_err(|_| {
        malformed(
            &format!("signal {sig_str:?} is not 2 hex chars"),
            full_frame,
        )
    })?;

    Ok(IncomingFrame::MotorVoltageRec {
        addr,
        voltage: MotorVoltage { centivolts, signal },
    })
}

/// Parse an error-response frame `!<addr>E<xx>;` (the `E` has already
/// been consumed).
fn parse_error_rec(addr: MotorAddress, payload: &[u8], full_frame: &[u8]) -> Result<IncomingFrame> {
    if payload.is_empty() {
        return Err(malformed("empty error payload", full_frame));
    }
    let code_str = std::str::from_utf8(payload)
        .map_err(|_| malformed("error code bytes are not valid UTF-8", full_frame))?;
    Ok(IncomingFrame::HubError {
        addr,
        code: HubErrorCode::from_wire(code_str),
    })
}

/// Parse a zero-padded 3-digit decimal byte (`000`..=`999`) as a `u8`.
/// Rejects out-of-range values so callers get a clean error rather
/// than a wraparound.
fn parse_three_digit(digits: &[u8], full_frame: &[u8]) -> Result<u8> {
    if digits.len() != 3 || !digits.iter().all(u8::is_ascii_digit) {
        return Err(malformed(
            &format!("expected 3 decimal digits, got {digits:?}"),
            full_frame,
        ));
    }
    let value = u16::from(digits[0] - b'0') * 100
        + u16::from(digits[1] - b'0') * 10
        + u16::from(digits[2] - b'0');
    u8::try_from(value).map_err(|_| {
        malformed(
            &format!("3-digit value {value} does not fit in u8"),
            full_frame,
        )
    })
}

/// Decode a UTF-8 string or produce a [`Error::Malformed`] pointing at
/// the full frame it came from.
fn string_from_utf8(bytes: &[u8], full_frame: &[u8]) -> Result<String> {
    std::str::from_utf8(bytes)
        .map(str::to_owned)
        .map_err(|_| malformed("string payload is not valid UTF-8", full_frame))
}

/// Build an [`Error::Malformed`] with the given detail and the full
/// frame's bytes (best-effort string-ified) attached as context.
fn malformed(detail: &str, frame: &[u8]) -> Error {
    Error::Malformed {
        detail: detail.to_owned(),
        raw: String::from_utf8_lossy(frame).into_owned(),
    }
}
