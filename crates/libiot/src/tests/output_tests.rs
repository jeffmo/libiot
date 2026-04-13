//! Tests for the output formatting module — [`OutputFormat`] parsing,
//! display round-tripping, and view-struct serialization shapes.

use std::str::FromStr;

use crate::output::AliasView;
use crate::output::CargoResultView;
use crate::output::ListAllView;
use crate::output::OutputFormat;

/// `OutputFormat::from_str("human")` parses to `Human`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn from_str_human_lowercase() {
    let fmt = OutputFormat::from_str("human").unwrap();
    assert_eq!(fmt, OutputFormat::Human);
}

/// `OutputFormat::from_str("JSON")` parses to `Json` (case-insensitive).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn from_str_json_uppercase() {
    let fmt = OutputFormat::from_str("JSON").unwrap();
    assert_eq!(fmt, OutputFormat::Json);
}

/// `OutputFormat::from_str("xml")` returns an error.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn from_str_unknown_format_errors() {
    let result = OutputFormat::from_str("xml");
    assert!(result.is_err());
    let msg = result.unwrap_err();
    assert!(
        msg.contains("xml"),
        "error message should mention the bad input, got: {msg}",
    );
}

/// `OutputFormat::default()` is `Human`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn default_is_human() {
    assert_eq!(OutputFormat::default(), OutputFormat::Human);
}

/// Round-trip: `format!("{}", OutputFormat::Human)` produces `"human"`,
/// and `format!("{}", OutputFormat::Json)` produces `"json"`.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn display_round_trip() {
    assert_eq!(format!("{}", OutputFormat::Human), "human");
    assert_eq!(format!("{}", OutputFormat::Json), "json");

    // And the round-trip back through FromStr:
    let human = OutputFormat::from_str(&format!("{}", OutputFormat::Human)).unwrap();
    assert_eq!(human, OutputFormat::Human);
    let json = OutputFormat::from_str(&format!("{}", OutputFormat::Json)).unwrap();
    assert_eq!(json, OutputFormat::Json);
}

/// `AliasView` serializes to the expected JSON shape with `alias` and
/// `target` fields.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn alias_view_serializes_correctly() {
    let view = AliasView {
        alias: "shades",
        target: "rollease-automate-pulse-pro-hub",
    };
    let json: serde_json::Value = serde_json::to_value(&view).expect("AliasView is serializable");
    assert_eq!(json["alias"], "shades");
    assert_eq!(json["target"], "rollease-automate-pulse-pro-hub");
    // Exactly two keys, no extras.
    assert_eq!(json.as_object().map(serde_json::Map::len), Some(2));
}

/// `ListAllView` serializes with both `clis` and `aliases` fields
/// present.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn list_all_view_serializes_with_both_fields() {
    let view = ListAllView {
        clis: vec![],
        aliases: vec![AliasView {
            alias: "test",
            target: "target",
        }],
    };
    let json: serde_json::Value = serde_json::to_value(&view).expect("ListAllView is serializable");
    assert!(json["clis"].is_array(), "clis should be an array");
    assert!(json["aliases"].is_array(), "aliases should be an array");
    assert_eq!(json["clis"].as_array().map(Vec::len), Some(0));
    assert_eq!(json["aliases"].as_array().map(Vec::len), Some(1));
}

/// `CargoResultView` skips `None` fields (`cargo_output` and
/// `alias_created`) in serialization.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn cargo_result_view_skips_none_fields() {
    let view = CargoResultView {
        ok: true,
        crate_name: "libiot-test",
        cargo_output: None,
        alias_created: None,
    };
    let json: serde_json::Value =
        serde_json::to_value(&view).expect("CargoResultView is serializable");
    let obj = json.as_object().expect("should be an object");
    assert!(
        !obj.contains_key("cargo_output"),
        "None fields should be skipped"
    );
    assert!(
        !obj.contains_key("alias_created"),
        "None fields should be skipped"
    );
    assert_eq!(obj.len(), 2, "only ok and crate_name should be present");

    // And verify that Some values do appear:
    let view_with = CargoResultView {
        ok: false,
        crate_name: "libiot-test",
        cargo_output: Some("some output"),
        alias_created: Some("test-alias"),
    };
    let json_with: serde_json::Value =
        serde_json::to_value(&view_with).expect("CargoResultView is serializable");
    let obj_with = json_with.as_object().expect("should be an object");
    assert!(obj_with.contains_key("cargo_output"));
    assert!(obj_with.contains_key("alias_created"));
    assert_eq!(obj_with.len(), 4);
}
