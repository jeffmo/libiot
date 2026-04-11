//! Async Rust client for the Rollease Acmeda Automate Pulse Pro shade
//! hub, speaking the local LAN ASCII protocol on TCP port 1487. No
//! auth, no cloud, no TLS.
//!
//! Part of the [`libiot`](https://github.com/jeffmo/libiot) workspace.
//!
//! This crate is currently scaffolding. The domain types, pure codec
//! layer, generic transport layer, and public client struct land in
//! subsequent commits. The crate-level usage examples and References
//! section will be added alongside the public client so that every
//! example compiles against real API surface.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
