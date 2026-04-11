//! Root of the non-codec test tree. Each `*_tests.rs` file here
//! exercises one subject file from `src/*.rs`. Codec-specific tests
//! live under `src/codec/tests/` instead.

mod automate_pulse_pro_hub_tests;
mod error_tests;
mod motor_address_tests;
mod motor_type_tests;
mod motor_voltage_tests;
mod transport_tests;
