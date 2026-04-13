---
name: iot-crate-standards
description: Enforce libiot workspace crate conventions — naming, architecture layering, module and file organization, constructor conventions, error-type patterns, import style, rustdoc and doctest expectations, test file layout, authoritative spec reference rules, and keyword enforcement. Triggered whenever code in the libiot repo is being added, modified, or reviewed. Use when adding a new crate, implementing device clients, refactoring existing code, writing tests, or running code reviews. Also applies when editing any `.rs` file under `crates/` or any `Cargo.toml` in the workspace.
---

# iot-crate-standards

These are the conventions every `libiot-*` crate must follow. Read this
skill before writing or reviewing any code in the workspace. When any
rule below is violated in code you're changing, fix it as part of your
change (unless explicitly asked not to).

## 1. Crate naming and shape

- **Library crates:** `libiot-<vendor>-<device>`
  - e.g. `libiot-rollease-automate-pulse-pro-hub`,
    `libiot-samsung-frametv`, `libiot-lutron-radiora2`
- **CLI crates (optional per device):** `libiot-<vendor>-<device>-cli`
  - Depends on the corresponding library crate and wraps it in an
    ergonomic command-line interface.
- Every `Cargo.toml` in the workspace **must** include `"libiot"` in its
  `keywords` array. CI enforces this via
  `scripts/check-libiot-keywords.sh`. When adding a new crate, put
  `"libiot"` first in the keywords list so it's impossible to miss.
- Each crate has its own `README.md` and `project-tracker.md` at its
  root.

## 2. Constructor convention

- If the client maintains a long-lived connection to the device (TCP,
  WebSocket, serial, etc.), the constructor is **`::connect(addr)`** and
  is `async`.
- If the client is stateless or connectionless (fire-and-forget UDP,
  per-request HTTP), the constructor is **`::new(addr)`** and is sync or
  async as appropriate.
- Prefer exactly one constructor. Builders are acceptable only when
  device configuration is genuinely non-trivial.

## 3. Three-layer architecture (MANDATORY)

Every `libiot-*` library crate is organized into three layers:

1. **Pure codec layer** — frame/packet encoding and decoding.
   - Zero `async`, zero `tokio`, zero I/O imports.
   - 100% unit-test coverage target.
   - Functions return `Vec<u8>`, `Result<T, Error>`, etc.
2. **Generic transport layer** — reads/writes bytes on an abstract
   stream.
   - Parameterized over `tokio::io::AsyncRead + AsyncWrite + Unpin`
     (or the equivalent trait for the device's transport).
   - Tested with `tokio::io::duplex()` pairs or equivalent in-memory
     fakes. No real sockets in unit tests.
3. **Thin public client layer** — the `XyzHub`/`XyzTv`/etc. struct.
   - Holds a concrete transport (e.g. `Transport<TcpStream>`) behind
     a `tokio::sync::Mutex`.
   - Each public method encodes a command, writes it through the
     transport, reads expected response frames, and returns typed data.
     The logic is thin — heavy lifting lives in the codec and
     transport layers.

This layering makes ~95% of each crate testable with nothing but
`#[test]` functions and in-memory fake streams. It's the single most
important organizational rule in the workspace.

## 4. Module and file organization

- **One public struct, trait, or enum per file**, named after the type
  in snake_case. Example: `struct AutomatePulseProHub` lives in
  `automate_pulse_pro_hub.rs`; `enum MotorType` lives in `motor_type.rs`.
- **Exception:** a tiny companion type (2-3 variant enum, simple helper
  struct) that's only used alongside one parent type can live in the
  same file as the parent when splitting would be more awkward than
  useful. Example: `struct Motor` + `enum MotorType` can share a file.
- **All modules are crate-private impl modules.** The public API is
  exposed exclusively through `pub use` re-exports in `lib.rs` (or
  `mod.rs` for nested modules). Never `pub mod foo` in an impl module.
- Within a module directory like `codec/`, the `mod.rs` file does the
  re-exporting and declares sub-modules. Impl modules are private to
  `codec`; `codec`'s surface to the rest of the crate is just what
  `mod.rs` re-exports.

## 5. Import statement style

- **Never** use compound `use` statements with curly braces.
- Always one symbol per line.
- Always sort `use` statements alphabetically.
- Never use `super::` in imports — always `crate::...` rooted paths.
- Group `std` → external crates → `crate::...` with blank lines between
  groups (rustfmt's `group_imports = "StdExternalCrate"` enforces this
  under nightly).

Correct:
```rust
use std::sync::Arc;

use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::codec::IncomingFrame;
use crate::codec::encode_open;
use crate::error::Error;
```

Incorrect:
```rust
use std::sync::Arc;
use tokio::{net::TcpStream, sync::Mutex};
use super::codec::{encode_open, IncomingFrame};
use crate::error::Error;
```

## 6. Formatting and layout

- Line length: 100 columns. Stay as close to 100 as readability allows
  when a line cannot fit.
- Always put the full `if`/`match` keyword + condition + opening `{` on
  a single line when the whole thing fits in 100 cols. If the opening
  `{` is the only thing that pushes it over, keep it on the same line
  anyway (rare exception to the 100-col rule).
- Match arms always end with a trailing comma, including block arms.
- Enum variants are defined in alphabetical order.
- Never place an opening `{` or `(` on its own line — always at the end
  of the previous line.

## 7. Error types

- Use the `thiserror::Error` derive macro for every custom error type.
- Errors are enums with detailed, distinct variants — one per failure
  mode, carrying relevant context (address, bytes received, parser
  detail, etc.).
- Every variant has an `#[error("...")]` attribute with a clear,
  human-readable message.
- Define a crate-local `type Result<T> = std::result::Result<T, MyError>;`
  alias so crate internals don't have to spell out the error type.
- Never use `Box<dyn Error>` or `anyhow::Error` on public API.

Example:
```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("I/O error talking to hub: {0}")]
    Io(#[from] std::io::Error),

    #[error("invalid motor address {input:?}: must be 3 chars [0-9A-Za-z]")]
    InvalidAddress { input: String },

    #[error("hub reported error {code:?} for motor {address}")]
    HubError { address: MotorAddress, code: HubErrorCode },
}

pub type Result<T> = std::result::Result<T, Error>;
```

## 8. No unwrap/expect in library code

- `unwrap()` and `expect()` are forbidden in library code. Return
  `Result` and let the caller decide.
- They're fine in `#[test]` functions and in truly-infallible const
  contexts.

## 9. Boolean literal parameter comment convention

When calling a function with a literal `true` or `false`, always prefix
the boolean literal with an inline comment naming the parameter:

```rust
parse_frames(input, /* strict = */ true);
writer.write_all(bytes, /* fsync = */ false);
```

This keeps the call site readable at a glance — a bare `true`/`false`
at a call site forces readers to jump to the function signature to
know what it means.

## 10. Authoritative spec references

Not every IoT device has an authoritative local-protocol spec reference.
When one **is** available (vendor PDF, community-maintained protocol
doc, RFC, spec site):

- **Every crate's top-level rustdoc** (`//! ...` in `lib.rs`) must link
  to the authoritative reference(s) in a "References" section near the
  top of the doc comment.
- **Any code that deliberately deviates from the spec** (e.g. because
  the spec is wrong and the field-verified behavior differs) must have
  a comment pointing at the exact section of the spec and explaining
  why the code does something different.
- Code review (`/iot-protocol-review`) explicitly checks "if an
  authoritative reference is available, do the encoded/decoded bytes
  match it?"

When no authoritative spec exists (pure reverse-engineering against a
black-box device):

- The crate's rustdoc and README should say so explicitly.
- Cite whatever community references, reverse-engineered libraries, or
  packet captures informed the implementation.

## 11. Crate-level rustdoc

- `lib.rs` opens with a `//!` doc block containing:
  - One-paragraph description of what the crate does.
  - At least 1, preferably 2-3, usage examples as code blocks.
  - A "References" section linking authoritative sources if available,
    or stating their absence and citing community references if not.
- CI runs `cargo doc --workspace --no-deps` under
  `RUSTDOCFLAGS="-D warnings"`, so broken intra-doc links fail the
  build.

## 12. Doctests

- Include doctests for public API examples wherever they're reasonably
  useful. `cargo test` runs them automatically.
- If a doctest would require a live network connection, mark it
  `no_run` and still include it — the readability of the example is
  more valuable than the runtime check.

## 13. Unit test organization

- **Never** use inline `#[cfg(test)] mod tests` at the bottom of an
  impl file. Tests live in sibling files.
- Tests for `src/foo.rs` live in `src/tests/foo_tests.rs`.
- Tests for `src/foo/bar.rs` live in `src/foo/tests/bar_tests.rs`.
- Tests for `src/foo/bar/baz.rs` live in `src/foo/bar/tests/baz_tests.rs`.
- The tests submodule is declared with `#[cfg(test)] mod tests;` in
  the adjacent `mod.rs` (or `lib.rs` at the crate root).
- Inside `tests/mod.rs`, declare each test file: `mod foo_tests;`,
  `mod bar_tests;`, etc.
- Test file naming: always `*_tests.rs`.

### Test content rules

- Every newly-added or updated test MUST include a clear, well-
  structured English description of what the test validates, placed
  as a doc comment immediately above the `#[test]` function.
- Tests written by Claude Code should note "Written by Claude Code,
  reviewed by a human." in that same doc comment.
- Tests must cover: happy paths, every documented error response,
  partial input, concatenated input, malformed input, timeouts, and
  every example frame captured in any authoritative reference the
  crate has access to.

## 14. `missing_docs` and `forbid(unsafe_code)`

Every crate's `lib.rs` starts with:

```rust
#![forbid(unsafe_code)]
#![warn(missing_docs)]
```

## 15. project-tracker.md (every crate has one)

See the `update-project-trackers` skill for full format details. Every
crate has a `project-tracker.md` at its root. When adding a new feature,
fixing a bug, or identifying follow-up work, update the appropriate
tracker. Project trackers are deliberately terse — sacrifice grammar
for concision — and end with an "Unresolved Questions" section when
relevant.

## 16. CLI crate one-liner title

Every `-cli` crate must have a single-sentence title as its clap
`about` string. This title:

- Is the **first line** of `<binary> --help` output.
- Must be a complete, standalone sentence (no trailing period).
- Must describe what the CLI controls, not implementation details.
- Is extracted at completion-generation time by `libiot completions`
  and shown as the tab-completion description for that CLI.

Example (in `cli.rs`):

```rust
#[command(
    about = "Control a Rollease Acmeda Automate Pulse Pro shade hub from the command line"
)]
```

Longer description text goes in `long_about` or the doc comment, not
in `about`. The one-liner must stand alone — it's the only thing
users see in tab-completion menus.

Code review (`/review-rust-api`) explicitly checks: does the `-cli`
crate have a concise `about` string that would read well as a
one-line completion description?
