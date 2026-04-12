#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Unified CLI dispatcher for the libiot ecosystem.
//!
//! `libiot` discovers all installed `libiot-*` CLI binaries on `$PATH`
//! and exposes them as subcommands, with alias management, per-command
//! environment variable injection, and `cargo install`/`uninstall`
//! wrappers.

fn main() {
    todo!("pre-parse dispatch not yet implemented")
}
