//! The [`MotorVoltage`] struct — parsed motor voltage reply.

/// A parsed voltage-query reply for one motor.
///
/// The hub reports motor battery voltage with two-decimal precision in a
/// fixed-point integer field of 5 digits. For example, `01208` means
/// `12.08` volts. To avoid floating-point rounding at the wire boundary
/// the raw centivolt value is stored; [`MotorVoltage::volts`] converts
/// on demand when a float is actually needed.
///
/// Wire format:
///
/// ```text
/// !<addr>pVc<NNNNN>,R<sig2>;
/// ```
///
/// The `,R<sig2>` trailer is the same undocumented signal-strength byte
/// that appears on position replies — see [`crate::MotorPosition::signal`]
/// for discussion.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MotorVoltage {
    /// Raw voltage × 100 as reported by the hub. `1208` means `12.08 V`.
    pub centivolts: u32,

    /// Undocumented signal-strength trailer byte — see
    /// [`crate::MotorPosition::signal`] for details.
    pub signal: u8,
}

impl MotorVoltage {
    /// Return the voltage as a floating-point number of volts.
    ///
    /// This is a convenience for logging and UI display. Precision-
    /// sensitive consumers should use [`MotorVoltage::centivolts`]
    /// directly.
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // u32 -> f32 is safe for realistic voltages
    pub fn volts(&self) -> f32 {
        self.centivolts as f32 / 100.0
    }
}
