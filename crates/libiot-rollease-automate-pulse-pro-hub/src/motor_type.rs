//! The [`MotorType`] enum — motor hardware class as reported by the hub.

/// Motor hardware class, as reported by the hub in version-query replies.
///
/// The Automate Pulse Pro hub reports a single ASCII character for each
/// paired device's motor class. The full mapping is drawn from the
/// [`aiopulse2`](https://github.com/sillyfrog/aiopulse2) reference
/// implementation (`aiopulse2/const.py`).
///
/// `HubGateway` represents the hub itself rather than a paired motor. The
/// hub reports itself in response to broadcast enumeration queries; the
/// public client API filters it out of motor listings automatically.
///
/// # Wire-format mapping
///
/// | Byte | Variant       | Notes                                     |
/// | ---- | ------------- | ----------------------------------------- |
/// | `A`  | `Ac`          | AC motor                                  |
/// | `B`  | `HubGateway`  | The hub itself (filtered from listings)   |
/// | `C`  | `Curtain`     | Curtain motor                             |
/// | `D`  | `Dc`          | DC motor                                  |
/// | `d`  | `DcLower`     | DC motor, lower-case variant              |
/// | `L`  | `Light`       | Lighting device                           |
/// | `S`  | `Socket`      | Switched socket                           |
/// | `U`  | `DcU`         | DC motor, `U` variant                     |
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MotorType {
    /// AC motor (`A`).
    Ac,
    /// Curtain motor (`C`).
    Curtain,
    /// DC motor (`D`).
    Dc,
    /// DC motor, lower-case variant (`d`).
    DcLower,
    /// DC motor, `U` variant (`U`).
    DcU,
    /// The hub itself (`B`). Filtered from motor listings.
    HubGateway,
    /// Lighting device (`L`).
    Light,
    /// Switched socket (`S`).
    Socket,
}

impl MotorType {
    /// Parse a single wire-format byte into a [`MotorType`]. Returns `None`
    /// for any byte the protocol does not document — callers should treat
    /// an unknown byte as a parse error, not silently skip it.
    #[must_use]
    pub fn from_wire_byte(byte: u8) -> Option<Self> {
        match byte {
            b'A' => Some(Self::Ac),
            b'B' => Some(Self::HubGateway),
            b'C' => Some(Self::Curtain),
            b'D' => Some(Self::Dc),
            b'd' => Some(Self::DcLower),
            b'L' => Some(Self::Light),
            b'S' => Some(Self::Socket),
            b'U' => Some(Self::DcU),
            _ => None,
        }
    }

    /// Return the single ASCII byte the hub uses to identify this motor
    /// type on the wire.
    #[must_use]
    pub fn wire_byte(&self) -> u8 {
        match self {
            Self::Ac => b'A',
            Self::Curtain => b'C',
            Self::Dc => b'D',
            Self::DcLower => b'd',
            Self::DcU => b'U',
            Self::HubGateway => b'B',
            Self::Light => b'L',
            Self::Socket => b'S',
        }
    }

    /// Return `true` if this motor type represents the hub itself rather
    /// than a paired motor. The public client API uses this to filter
    /// motor listings.
    #[must_use]
    pub fn is_hub_gateway(&self) -> bool {
        matches!(self, Self::HubGateway)
    }
}
