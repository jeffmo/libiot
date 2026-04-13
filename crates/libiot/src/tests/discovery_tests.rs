//! Tests for the PATH scanning logic in [`crate::discovery`].

use std::fs;
use std::path::Path;

use crate::discovery::discover_clis_with_path;
use crate::discovery::find_cli_with_path;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a file inside `dir` with the given name. On Unix, optionally
/// set it executable.
fn create_file(dir: &Path, name: &str, executable: bool) {
    let path = dir.join(name);
    fs::write(&path, "#!/bin/sh\n").unwrap();
    if executable {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
    } else {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        }
    }
}

/// Build a PATH string from a slice of directory paths, joined with the
/// platform separator.
fn make_path(dirs: &[&Path]) -> String {
    std::env::join_paths(dirs).unwrap().into_string().unwrap()
}

// ---------------------------------------------------------------------------
// discover_clis_with_path
// ---------------------------------------------------------------------------

/// An empty PATH string should produce no results.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn empty_path_returns_empty_vec() {
    let result = discover_clis_with_path("");
    assert!(result.is_empty());
}

/// A directory containing `libiot-foo` and `libiot-bar` returns both
/// entries, sorted alphabetically by short name.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn discovers_two_clis_sorted() {
    let tmp = tempfile::tempdir().unwrap();
    create_file(tmp.path(), "libiot-foo", true);
    create_file(tmp.path(), "libiot-bar", true);

    let path = make_path(&[tmp.path()]);
    let result = discover_clis_with_path(&path);

    assert_eq!(result.len(), 2);
    assert_eq!(result[0].name, "bar");
    assert_eq!(result[1].name, "foo");
    assert_eq!(result[0].path, tmp.path().join("libiot-bar"));
    assert_eq!(result[1].path, tmp.path().join("libiot-foo"));
}

/// Files that do not start with `libiot-` are ignored.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn non_libiot_prefix_files_are_ignored() {
    let tmp = tempfile::tempdir().unwrap();
    create_file(tmp.path(), "libiot-real", true);
    create_file(tmp.path(), "not-libiot-fake", true);
    create_file(tmp.path(), "cargo", true);

    let path = make_path(&[tmp.path()]);
    let result = discover_clis_with_path(&path);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "real");
}

/// When the same short name appears in two PATH directories, the first
/// directory's entry wins (matching Unix shell resolution order).
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn deduplication_first_path_wins() {
    let dir1 = tempfile::tempdir().unwrap();
    let dir2 = tempfile::tempdir().unwrap();

    create_file(dir1.path(), "libiot-dup", true);
    create_file(dir2.path(), "libiot-dup", true);

    let path = make_path(&[dir1.path(), dir2.path()]);
    let result = discover_clis_with_path(&path);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "dup");
    assert_eq!(
        result[0].path,
        dir1.path().join("libiot-dup"),
        "first PATH directory should win",
    );
}

/// Non-existent directories on PATH are silently ignored.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn nonexistent_path_dirs_are_ignored() {
    let tmp = tempfile::tempdir().unwrap();
    create_file(tmp.path(), "libiot-ok", true);

    let bogus = tmp.path().join("does-not-exist");
    let path = make_path(&[&bogus, tmp.path()]);
    let result = discover_clis_with_path(&path);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "ok");
}

// ---------------------------------------------------------------------------
// find_cli_with_path
// ---------------------------------------------------------------------------

/// `find_cli_with_path` returns `Some` when `libiot-foo` exists.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn find_cli_returns_some_when_present() {
    let tmp = tempfile::tempdir().unwrap();
    create_file(tmp.path(), "libiot-foo", true);

    let path = make_path(&[tmp.path()]);
    let result = find_cli_with_path("foo", &path);

    assert_eq!(result, Some(tmp.path().join("libiot-foo")));
}

/// `find_cli_with_path` returns `None` when the target does not exist.
///
/// Written by Claude Code, reviewed by a human.
#[test]
fn find_cli_returns_none_when_absent() {
    let tmp = tempfile::tempdir().unwrap();
    create_file(tmp.path(), "libiot-other", true);

    let path = make_path(&[tmp.path()]);
    let result = find_cli_with_path("nonexistent", &path);

    assert!(result.is_none());
}

// ---------------------------------------------------------------------------
// Unix-specific: executable bit
// ---------------------------------------------------------------------------

/// On Unix, files without the executable bit are skipped even if their
/// name matches the `libiot-*` pattern.
///
/// Written by Claude Code, reviewed by a human.
#[cfg(unix)]
#[test]
fn non_executable_files_are_skipped() {
    let tmp = tempfile::tempdir().unwrap();
    create_file(tmp.path(), "libiot-noexec", false);
    create_file(tmp.path(), "libiot-exec", true);

    let path = make_path(&[tmp.path()]);
    let result = discover_clis_with_path(&path);

    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "exec");
}

/// On Unix, `find_cli_with_path` skips non-executable matches and
/// returns `None` when no executable candidate exists.
///
/// Written by Claude Code, reviewed by a human.
#[cfg(unix)]
#[test]
fn find_cli_skips_non_executable() {
    let tmp = tempfile::tempdir().unwrap();
    create_file(tmp.path(), "libiot-noexec", false);

    let path = make_path(&[tmp.path()]);
    let result = find_cli_with_path("noexec", &path);

    assert!(result.is_none());
}
