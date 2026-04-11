//! Exhaustive tests for [`crate::codec::parse_frames`].
//!
//! Coverage targets:
//!
//! - Every [`IncomingFrame`] variant has at least one happy-path test.
//! - Every real-hub capture from §6 of `PULSE_PRO_LOCAL_API.md` appears
//!   as a test fixture — these are the highest-signal regression
//!   guards in the crate.
//! - Partial-frame handling across multiple calls.
//! - Concatenated-frame handling in one call.
//! - Every documented `HubErrorCode` variant plus an unknown one.
//! - Malformed input returns an error rather than panicking.

use crate::codec::incoming_frame::IncomingFrame;
use crate::codec::parser::parse_frames;
use crate::error::Error;
use crate::error::HubErrorCode;
use crate::motor_address::MotorAddress;
use crate::motor_type::MotorType;

fn addr(s: &str) -> MotorAddress {
    MotorAddress::new(s).expect("test address is valid")
}

// -- single-frame happy paths ---------------------------------------------

/// Verifies the parser recognizes a hub-name reply
/// (`!000NAME<string>;`) and returns the string payload exactly as
/// emitted by the hub. Real-hub capture from §6 of
/// `PULSE_PRO_LOCAL_API.md`: `!000NAME6217 Shade Hub;`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parser_recognizes_hub_name_reply() {
    let mut buf = b"!000NAME6217 Shade Hub;".to_vec();
    let frames = parse_frames(&mut buf).unwrap();
    assert_eq!(
        frames,
        vec![IncomingFrame::HubName("6217 Shade Hub".into())]
    );
    assert!(buf.is_empty(), "buffer should be drained");
}

/// Verifies the parser recognizes a hub-serial reply. Real-hub
/// capture from §6: `!000SN2016197;`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parser_recognizes_hub_serial_reply() {
    let mut buf = b"!000SN2016197;".to_vec();
    let frames = parse_frames(&mut buf).unwrap();
    assert_eq!(frames, vec![IncomingFrame::HubSerial("2016197".into())]);
}

/// Verifies the parser recognizes a motor friendly-name reply,
/// including names that contain spaces like `"Dining Room"`. Real-hub
/// capture from §6: `!MWXNAMEDining Room;`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parser_recognizes_motor_name_with_spaces() {
    let mut buf = b"!MWXNAMEDining Room;".to_vec();
    let frames = parse_frames(&mut buf).unwrap();
    assert_eq!(
        frames,
        vec![IncomingFrame::MotorName {
            addr: addr("MWX"),
            name: "Dining Room".into(),
        }],
    );
}

/// Verifies the parser recognizes a motor version reply and correctly
/// maps the ASCII type byte `D` to [`MotorType::Dc`]. Real-hub
/// capture from §6: `!4JKvD22;` → DC motor, firmware v2.2.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parser_recognizes_motor_version_reply_with_dc_type() {
    let mut buf = b"!4JKvD22;".to_vec();
    let frames = parse_frames(&mut buf).unwrap();
    assert_eq!(
        frames,
        vec![IncomingFrame::MotorVersionRec {
            addr: addr("4JK"),
            motor_type: MotorType::Dc,
            version: "22".into(),
        }],
    );
}

/// Verifies the parser recognizes the hub's self-identification frame
/// with type byte `B` mapping to [`MotorType::HubGateway`]. Real-hub
/// capture from §6: `!BR1vB10;`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parser_recognizes_hub_self_identification_frame() {
    let mut buf = b"!BR1vB10;".to_vec();
    let frames = parse_frames(&mut buf).unwrap();
    assert_eq!(
        frames,
        vec![IncomingFrame::MotorVersionRec {
            addr: addr("BR1"),
            motor_type: MotorType::HubGateway,
            version: "10".into(),
        }],
    );
}

/// Verifies the parser correctly extracts the 3-digit closed-percent,
/// 3-digit tilt, and 2-hex signal-strength trailer from a position
/// reply, including the documented quirk that some motors legitimately
/// report tilt values above 100. Real-hub capture from §6:
/// `!MWXr100b180,R4C;` — fully closed, tilt 180, signal 0x4C.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parser_extracts_position_fields_with_tilt_above_100_and_signal_byte() {
    let mut buf = b"!MWXr100b180,R4C;".to_vec();
    let frames = parse_frames(&mut buf).unwrap();
    match &frames[..] {
        [IncomingFrame::MotorPositionRec { addr: a, position }] => {
            assert_eq!(*a, addr("MWX"));
            assert_eq!(position.closed_percent, 100);
            assert_eq!(position.tilt_percent, 180);
            assert_eq!(position.signal, 0x4C);
        },
        other => panic!("expected one MotorPositionRec, got {other:?}"),
    }
}

/// Verifies the parser handles a fully-open position reply with the
/// higher-signal `R58` trailer. Real-hub capture from §6:
/// `!4JKr000b000,R58;`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parser_extracts_fully_open_position_with_high_signal() {
    let mut buf = b"!4JKr000b000,R58;".to_vec();
    let frames = parse_frames(&mut buf).unwrap();
    match &frames[..] {
        [IncomingFrame::MotorPositionRec { addr: a, position }] => {
            assert_eq!(*a, addr("4JK"));
            assert_eq!(position.closed_percent, 0);
            assert_eq!(position.tilt_percent, 0);
            assert_eq!(position.signal, 0x58);
        },
        other => panic!("expected one MotorPositionRec, got {other:?}"),
    }
}

/// Verifies the parser handles a voltage reply with 5-digit centivolt
/// precision and a signal-strength trailer. Real-hub capture from §6:
/// `!4JKpVc01208,R58;` — 12.08 V, signal 0x58.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parser_extracts_voltage_reply_with_centivolts_and_signal() {
    let mut buf = b"!4JKpVc01208,R58;".to_vec();
    let frames = parse_frames(&mut buf).unwrap();
    match &frames[..] {
        [IncomingFrame::MotorVoltageRec { addr: a, voltage }] => {
            assert_eq!(*a, addr("4JK"));
            assert_eq!(voltage.centivolts, 1208);
            assert_eq!(voltage.signal, 0x58);
            assert!((voltage.volts() - 12.08).abs() < 0.001);
        },
        other => panic!("expected one MotorVoltageRec, got {other:?}"),
    }
}

/// Verifies the parser recognizes a motor address-edit acknowledgement
/// (`!<addr>A;`) as [`IncomingFrame::AddressAck`], per §2.7 of
/// `PULSE_PRO_LOCAL_API.md`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parser_recognizes_address_edit_ack() {
    let mut buf = b"!4JKA;".to_vec();
    let frames = parse_frames(&mut buf).unwrap();
    assert_eq!(
        frames,
        vec![IncomingFrame::AddressAck { addr: addr("4JK") }]
    );
}

// -- concatenated and real-hub burst frames -------------------------------

/// Verifies the parser splits a concatenated version-query response
/// burst into four separate frames — one hub self-identification plus
/// three motor version records. Real-hub capture from §6:
/// `!BR1vB10;!4JKvD22;!MWXvD22;!3YCvD22;`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parser_splits_real_hub_version_query_burst() {
    let mut buf = b"!BR1vB10;!4JKvD22;!MWXvD22;!3YCvD22;".to_vec();
    let frames = parse_frames(&mut buf).unwrap();
    assert_eq!(frames.len(), 4);
    assert!(buf.is_empty());

    // Verify every motor type was parsed correctly.
    let types: Vec<_> = frames
        .iter()
        .filter_map(|f| match f {
            IncomingFrame::MotorVersionRec { motor_type, .. } => Some(*motor_type),
            _ => None,
        })
        .collect();
    assert_eq!(
        types,
        vec![
            MotorType::HubGateway,
            MotorType::Dc,
            MotorType::Dc,
            MotorType::Dc,
        ],
    );
}

/// Verifies the parser splits a concatenated position-query response
/// burst into three separate frames and preserves each motor's tilt
/// and signal bytes correctly. Real-hub capture from §6:
/// `!4JKr000b000,R58;!MWXr100b180,R4C;!3YCr000b000,R4C;`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parser_splits_real_hub_position_query_burst() {
    let mut buf = b"!4JKr000b000,R58;!MWXr100b180,R4C;!3YCr000b000,R4C;".to_vec();
    let frames = parse_frames(&mut buf).unwrap();
    assert_eq!(frames.len(), 3);

    // Sanity-check each frame's fields in the order they arrive.
    let positions: Vec<_> = frames
        .iter()
        .filter_map(|f| match f {
            IncomingFrame::MotorPositionRec { addr, position } => Some((*addr, *position)),
            _ => None,
        })
        .collect();

    assert_eq!(positions.len(), 3);

    assert_eq!(positions[0].0, addr("4JK"));
    assert_eq!(positions[0].1.closed_percent, 0);
    assert_eq!(positions[0].1.tilt_percent, 0);
    assert_eq!(positions[0].1.signal, 0x58);

    assert_eq!(positions[1].0, addr("MWX"));
    assert_eq!(positions[1].1.closed_percent, 100);
    assert_eq!(positions[1].1.tilt_percent, 180);
    assert_eq!(positions[1].1.signal, 0x4C);

    assert_eq!(positions[2].0, addr("3YC"));
    assert_eq!(positions[2].1.closed_percent, 0);
    assert_eq!(positions[2].1.tilt_percent, 0);
    assert_eq!(positions[2].1.signal, 0x4C);
}

/// Verifies the parser splits a concatenated motor-name reply burst
/// into three frames with mixed single-word and multi-word names.
/// Real-hub capture from §6:
/// `!4JKNAMEJohn House;!MWXNAMEDining Room;!3YCNAMEKitchen;`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parser_splits_real_hub_motor_name_query_burst() {
    let mut buf = b"!4JKNAMEJohn House;!MWXNAMEDining Room;!3YCNAMEKitchen;".to_vec();
    let frames = parse_frames(&mut buf).unwrap();

    let names: Vec<_> = frames
        .iter()
        .filter_map(|f| match f {
            IncomingFrame::MotorName { addr, name } => Some((*addr, name.clone())),
            _ => None,
        })
        .collect();

    assert_eq!(
        names,
        vec![
            (addr("4JK"), "John House".to_string()),
            (addr("MWX"), "Dining Room".to_string()),
            (addr("3YC"), "Kitchen".to_string()),
        ],
    );
}

// -- partial-frame handling -----------------------------------------------

/// Verifies that feeding the parser a byte sequence that ends mid-frame
/// leaves the partial tail in the buffer for the next call, without
/// dropping any bytes. This is the core of the firehose pattern
/// described in §2.9 quirk #1 of `PULSE_PRO_LOCAL_API.md`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parser_accumulates_partial_frames_across_calls() {
    // First half: complete hub-name + the start of a hub-serial frame.
    let mut buf = b"!000NAME6217 Shade Hub;!000SN".to_vec();
    let frames1 = parse_frames(&mut buf).unwrap();
    assert_eq!(
        frames1,
        vec![IncomingFrame::HubName("6217 Shade Hub".into())]
    );
    assert_eq!(buf, b"!000SN", "partial tail should remain");

    // Second half arrives on the next read.
    buf.extend_from_slice(b"2016197;");
    let frames2 = parse_frames(&mut buf).unwrap();
    assert_eq!(frames2, vec![IncomingFrame::HubSerial("2016197".into())]);
    assert!(buf.is_empty());
}

/// Verifies the parser returns an empty vector (and leaves the buffer
/// untouched) when given a buffer that does not contain any complete
/// frame.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parser_returns_empty_when_no_complete_frame_yet() {
    let mut buf = b"!000NAME6217 Shad".to_vec();
    let frames = parse_frames(&mut buf).unwrap();
    assert_eq!(frames, Vec::new());
    assert_eq!(buf, b"!000NAME6217 Shad");
}

/// Verifies the parser tolerates stray CR/LF bytes that sometimes
/// sneak in when the transport reassembles packets across TCP read
/// boundaries.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parser_tolerates_stray_cr_lf_bytes() {
    let mut buf = b"!000NAME6217\rShade\nHub;".to_vec();
    let frames = parse_frames(&mut buf).unwrap();
    // CR/LF bytes are stripped before classification, so the name
    // reads as a contiguous string.
    assert_eq!(frames, vec![IncomingFrame::HubName("6217ShadeHub".into())],);
}

// -- hub error codes ------------------------------------------------------

/// Verifies every documented hub error code is parsed into the
/// corresponding [`HubErrorCode`] variant per §2.8 of
/// `PULSE_PRO_LOCAL_API.md`, and that an undocumented code is captured
/// as [`HubErrorCode::Unknown`] rather than dropped.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parser_maps_every_documented_error_code_and_captures_unknowns() {
    let cases: &[(&[u8], HubErrorCode)] = &[
        (b"!4JKEbz;", HubErrorCode::Busy),
        (b"!4JKEdf;", HubErrorCode::TooManyMotors),
        (b"!4JKEnp;", HubErrorCode::NoSuchMotor),
        (b"!4JKEnc;", HubErrorCode::NoLimitsSet),
        (b"!4JKEmh;", HubErrorCode::MasterHallSensor),
        (b"!4JKEsh;", HubErrorCode::SlaveHallSensor),
        (b"!4JKEor;", HubErrorCode::ObstacleUp),
        (b"!4JKEcr;", HubErrorCode::ObstacleDown),
        (b"!4JKEpl;", HubErrorCode::LowVoltage),
        (b"!4JKEph;", HubErrorCode::HighVoltage),
        (b"!4JKEnl;", HubErrorCode::MotorOffline),
        (b"!4JKEec;", HubErrorCode::Generic),
        (b"!4JKEzz;", HubErrorCode::Unknown("zz".into())),
    ];

    for (wire, expected) in cases {
        let mut buf = wire.to_vec();
        let frames = parse_frames(&mut buf).unwrap();
        match &frames[..] {
            [IncomingFrame::HubError { addr: a, code }] => {
                assert_eq!(*a, addr("4JK"));
                assert_eq!(code, expected, "wire: {:?}", String::from_utf8_lossy(wire));
            },
            other => panic!(
                "expected HubError for wire {:?}, got {other:?}",
                String::from_utf8_lossy(wire)
            ),
        }
    }
}

// -- malformed input ------------------------------------------------------

/// Verifies that a frame missing the leading `!` surfaces as
/// [`Error::Malformed`] instead of panicking.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parser_rejects_frame_without_leading_exclamation() {
    let mut buf = b"000NAME6217 Shade Hub;".to_vec();
    match parse_frames(&mut buf) {
        Err(Error::Malformed { .. }) => {},
        other => panic!("expected Malformed, got {other:?}"),
    }
}

/// Verifies that an address containing a non-alphanumeric byte
/// surfaces as [`Error::Malformed`] instead of panicking or silently
/// passing.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parser_rejects_frame_with_non_alphanumeric_address() {
    let mut buf = b"!4J!o;".to_vec();
    match parse_frames(&mut buf) {
        Err(Error::Malformed { .. }) => {},
        other => panic!("expected Malformed, got {other:?}"),
    }
}

/// Verifies that a position frame missing the `,R<hex2>` trailer
/// surfaces as [`Error::Malformed`] rather than producing garbage
/// position fields.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parser_rejects_position_frame_missing_signal_trailer() {
    let mut buf = b"!4JKr000b000;".to_vec();
    match parse_frames(&mut buf) {
        Err(Error::Malformed { .. }) => {},
        other => panic!("expected Malformed, got {other:?}"),
    }
}

/// Verifies that a position frame whose closed-percent digits are
/// non-decimal (e.g. `0A0`) surfaces as [`Error::Malformed`].
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parser_rejects_position_frame_with_non_decimal_closed_digits() {
    let mut buf = b"!4JKr0A0b000,R58;".to_vec();
    match parse_frames(&mut buf) {
        Err(Error::Malformed { .. }) => {},
        other => panic!("expected Malformed, got {other:?}"),
    }
}

/// Verifies that a signal-strength field with non-hex bytes surfaces
/// as [`Error::Malformed`].
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parser_rejects_position_frame_with_non_hex_signal() {
    let mut buf = b"!4JKr000b000,RZZ;".to_vec();
    match parse_frames(&mut buf) {
        Err(Error::Malformed { .. }) => {},
        other => panic!("expected Malformed, got {other:?}"),
    }
}
