//! The [`MotorPosition`] struct — parsed motor position reply.

/// A parsed motor position reply.
///
/// The Pulse Pro hub returns a motor's current lift percentage, tilt
/// percentage, and an undocumented signal-strength trailer in a single
/// frame of the form:
///
/// ```text
/// !<addr>r<closed3>b<tilt3>,R<sig2>;
/// ```
///
/// Note the field widths — they do **not** match the vendor PDF, which
/// documents 2-digit percentages. The field-verified on-wire format is
/// 3 digits, confirmed against real hub firmware and against the
/// `aiopulse2` reference implementation. See §2.5 of the in-crate
/// `PULSE_PRO_LOCAL_API.md` for the full discussion.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MotorPosition {
    /// Closed-lift percentage. `0` = fully open, `100` = fully closed.
    pub closed_percent: u8,

    /// Tilt percentage — the angle of individual slats on venetian blinds
    /// (horizontal blinds) or shutters. `0` = slats horizontal/open
    /// (maximum light), `100` = slats fully angled/closed (minimum
    /// light). Tilt only applies to slatted shades; roller shades,
    /// cellular shades, and drapes silently ignore tilt commands and
    /// typically report `0` here.
    ///
    /// Most motors report values in `0..=100`, but some legitimately
    /// report values above 100 — a real hub captured a tilt of `180` on
    /// a Dining Room motor, so this field is `u16` to avoid truncation.
    pub tilt_percent: u16,

    /// Undocumented signal-strength trailer (`,R<hex2>` on the wire).
    ///
    /// Almost certainly represents the RF link quality on the 433 MHz ARC
    /// link between hub and motor — `aiopulse2` captures it under the
    /// name `signal`. Higher values appear to correlate with stronger
    /// links (e.g. `0x58` observed on a close motor vs `0x4C` on a
    /// weaker one), but the exact scale is not documented by Rollease.
    pub signal: u8,
}
