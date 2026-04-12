//! Tests for [`crate::hub_connection::parse_hub_spec`].

use crate::hub_connection::parse_hub_spec;

/// Verifies that a bare IPv4 address gets the default port 1487 appended.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_hub_spec_appends_default_port_to_bare_ipv4() {
    assert_eq!(
        parse_hub_spec("192.168.5.234").unwrap(),
        "192.168.5.234:1487"
    );
}

/// Verifies that an IPv4 address with an explicit port is returned unchanged.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_hub_spec_preserves_explicit_port_on_ipv4() {
    assert_eq!(
        parse_hub_spec("192.168.5.234:9999").unwrap(),
        "192.168.5.234:9999"
    );
}

/// Verifies that a hostname gets the default port appended.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_hub_spec_appends_default_port_to_hostname() {
    assert_eq!(parse_hub_spec("my-hub.local").unwrap(), "my-hub.local:1487");
}

/// Verifies that a hostname with an explicit port is returned unchanged.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_hub_spec_preserves_explicit_port_on_hostname() {
    assert_eq!(
        parse_hub_spec("my-hub.local:9999").unwrap(),
        "my-hub.local:9999"
    );
}

/// Verifies that a bracketed IPv6 address gets the default port appended.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_hub_spec_appends_default_port_to_bracketed_ipv6() {
    assert_eq!(parse_hub_spec("[::1]").unwrap(), "[::1]:1487");
}

/// Verifies that a bracketed IPv6 address with an explicit port is
/// returned unchanged.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_hub_spec_preserves_explicit_port_on_bracketed_ipv6() {
    assert_eq!(parse_hub_spec("[::1]:9999").unwrap(), "[::1]:9999");
}

/// Verifies that bare IPv6 (without brackets) is rejected, because the
/// colons in the address are ambiguous with the port separator.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_hub_spec_rejects_bare_ipv6_without_brackets() {
    assert!(parse_hub_spec("::1").is_err());
    assert!(parse_hub_spec("fe80::1").is_err());
}

/// Verifies that an empty string is rejected.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn parse_hub_spec_rejects_empty_input() {
    assert!(parse_hub_spec("").is_err());
    assert!(parse_hub_spec("   ").is_err());
}
