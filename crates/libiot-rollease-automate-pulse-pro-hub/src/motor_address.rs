//! The [`MotorAddress`] newtype and its construction helpers.

use std::fmt;
use std::str::FromStr;

use crate::error::Error;
use crate::error::Result;

/// A 3-character motor address as assigned by the hub at pairing time.
///
/// The Automate Pulse Pro hub assigns each paired motor a unique 3-character
/// address drawn from the alphabet `[0-9A-Za-z]`. Addresses are
/// **case-sensitive** on the wire — the hub treats `"4JK"` and `"4jk"` as
/// different motors. Construction via [`MotorAddress::new`] enforces both
/// the length and the alphabet.
///
/// The reserved address `"000"` is a broadcast target that matches every
/// paired motor. It is exposed as [`MotorAddress::BROADCAST`] and detected
/// via [`MotorAddress::is_broadcast`].
///
/// # Examples
///
/// ```
/// use libiot_rollease_automate_pulse_pro_hub::MotorAddress;
///
/// let kitchen = MotorAddress::new("3YC").expect("3YC is a valid address");
/// assert_eq!(kitchen.as_str(), "3YC");
/// assert!(!kitchen.is_broadcast());
///
/// assert!(MotorAddress::BROADCAST.is_broadcast());
/// assert_eq!(MotorAddress::BROADCAST.as_str(), "000");
///
/// // Addresses are case-sensitive — "4jk" and "4JK" are different motors.
/// assert_ne!(
///     MotorAddress::new("4JK").unwrap(),
///     MotorAddress::new("4jk").unwrap(),
/// );
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct MotorAddress([u8; 3]);

impl MotorAddress {
    /// The reserved broadcast address `000`.
    ///
    /// Sending a command to this address targets every paired motor on the
    /// hub. For queries, the hub replies with one frame per motor.
    pub const BROADCAST: MotorAddress = MotorAddress(*b"000");

    /// Construct a [`MotorAddress`] from a string slice.
    ///
    /// Returns [`Error::InvalidAddress`] if `s` is not exactly 3 characters
    /// of `[0-9A-Za-z]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use libiot_rollease_automate_pulse_pro_hub::MotorAddress;
    ///
    /// assert!(MotorAddress::new("4JK").is_ok());
    /// assert!(MotorAddress::new("3YC").is_ok());
    /// assert!(MotorAddress::new("000").is_ok());
    ///
    /// assert!(MotorAddress::new("4J").is_err());     // too short
    /// assert!(MotorAddress::new("4JKL").is_err());   // too long
    /// assert!(MotorAddress::new("4J!").is_err());    // bad char
    /// ```
    pub fn new(s: &str) -> Result<Self> {
        let bytes = s.as_bytes();
        if bytes.len() != 3 {
            return Err(Error::InvalidAddress {
                input: s.to_owned(),
            });
        }
        for &b in bytes {
            if !b.is_ascii_alphanumeric() {
                return Err(Error::InvalidAddress {
                    input: s.to_owned(),
                });
            }
        }
        let mut buf = [0u8; 3];
        buf.copy_from_slice(bytes);
        Ok(MotorAddress(buf))
    }

    /// Return the address as a borrowed ASCII string slice.
    ///
    /// The backing bytes are guaranteed to be valid ASCII alphanumeric
    /// by construction — [`MotorAddress::new`] is the only way to
    /// produce one — so this conversion is infallible in practice.
    ///
    /// # Panics
    ///
    /// This function contains an `.expect(...)` on the result of
    /// [`std::str::from_utf8`] that is unreachable given the
    /// construction invariants of [`MotorAddress`]. The only way to
    /// hit it would be to bypass the public constructors, which the
    /// crate does not allow (modules are `pub(crate)` at most). The
    /// crate forbids `unsafe_code`, so the safe infallible escape
    /// hatch `std::str::from_utf8_unchecked` is unavailable.
    #[must_use]
    pub fn as_str(&self) -> &str {
        std::str::from_utf8(&self.0).expect("MotorAddress bytes are valid ASCII by construction")
    }

    /// Return `true` if this is the reserved broadcast address `000`.
    #[must_use]
    pub fn is_broadcast(&self) -> bool {
        self.0 == *b"000"
    }

    /// Return the raw 3-byte representation. Intended for use by the codec
    /// layer when serializing frames; downstream users should prefer
    /// [`MotorAddress::as_str`].
    //
    // This method is intentionally unused at this point in the stack — the
    // codec layer that consumes it lands in a later commit. `#[expect]` is
    // used instead of `#[allow]` so that once the codec lands and the
    // method becomes used, the compiler emits a warning reminding us to
    // remove this attribute.
    #[expect(
        dead_code,
        reason = "consumed by the codec encoder that lands in a later commit"
    )]
    pub(crate) fn as_bytes(&self) -> &[u8; 3] {
        &self.0
    }
}

impl fmt::Debug for MotorAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "MotorAddress({:?})", self.as_str())
    }
}

impl fmt::Display for MotorAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for MotorAddress {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        MotorAddress::new(s)
    }
}

impl TryFrom<&str> for MotorAddress {
    type Error = Error;

    fn try_from(s: &str) -> Result<Self> {
        MotorAddress::new(s)
    }
}
