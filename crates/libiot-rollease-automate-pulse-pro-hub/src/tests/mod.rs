//! Root of the non-codec test tree. Each `*_tests.rs` file here
//! exercises one subject file from `src/*.rs`.
//!
//! The codec tests live separately under `src/codec/tests/` and are
//! declared from `src/codec/mod.rs` so that codec-layer and
//! domain-layer tests stay visually separated.

mod error_tests;
mod motor_address_tests;
mod motor_type_tests;
mod motor_voltage_tests;
mod transport_tests;
