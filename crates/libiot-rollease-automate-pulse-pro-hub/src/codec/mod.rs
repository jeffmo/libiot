//! Pure codec layer — frame encoding and decoding with zero I/O.
//!
//! This module and its children are intentionally free of any `async`,
//! any `tokio` imports, and any real I/O. Every function either builds
//! a `Vec<u8>` (encoder) or consumes a `&mut Vec<u8>` (parser), which
//! makes the entire codec testable with nothing but `#[test]`
//! functions. See the `iot-crate-standards` skill §3 for the rationale.

mod encoder;
mod incoming_frame;
mod parser;

#[cfg(test)]
mod tests;

pub(crate) use crate::codec::encoder::encode_close;
pub(crate) use crate::codec::encoder::encode_close_all;
#[cfg(feature = "dangerous-ops")]
pub(crate) use crate::codec::encoder::encode_delete_limits;
#[cfg(feature = "dangerous-ops")]
pub(crate) use crate::codec::encoder::encode_delete_motor;
#[cfg(feature = "dangerous-ops")]
pub(crate) use crate::codec::encoder::encode_factory_reset;
pub(crate) use crate::codec::encoder::encode_jog_close;
pub(crate) use crate::codec::encoder::encode_jog_open;
pub(crate) use crate::codec::encoder::encode_move_to;
pub(crate) use crate::codec::encoder::encode_open;
pub(crate) use crate::codec::encoder::encode_open_all;
#[cfg(feature = "dangerous-ops")]
pub(crate) use crate::codec::encoder::encode_pair_motor;
pub(crate) use crate::codec::encoder::encode_query_hub_name;
pub(crate) use crate::codec::encoder::encode_query_hub_serial;
pub(crate) use crate::codec::encoder::encode_query_motor_enum;
pub(crate) use crate::codec::encoder::encode_query_motor_name;
pub(crate) use crate::codec::encoder::encode_query_motor_position;
pub(crate) use crate::codec::encoder::encode_query_motor_position_all;
pub(crate) use crate::codec::encoder::encode_query_motor_version;
pub(crate) use crate::codec::encoder::encode_query_motor_voltage;
#[cfg(feature = "dangerous-ops")]
pub(crate) use crate::codec::encoder::encode_set_lower_limit;
#[cfg(feature = "dangerous-ops")]
pub(crate) use crate::codec::encoder::encode_set_upper_limit;
pub(crate) use crate::codec::encoder::encode_stop;
pub(crate) use crate::codec::encoder::encode_stop_all;
pub(crate) use crate::codec::encoder::encode_tilt;
#[cfg(feature = "dangerous-ops")]
pub(crate) use crate::codec::encoder::encode_unpair_motor;
pub(crate) use crate::codec::incoming_frame::IncomingFrame;
pub(crate) use crate::codec::parser::parse_frames;
