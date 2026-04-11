# libiot — Project Conventions

## What this is

`libiot` is a Cargo workspace housing a collection of Rust connector
libraries for consumer IoT devices. Each device family gets its own
library crate under `crates/`, optionally paired with a `-cli` binary
crate that wraps the library in an ergonomic command-line interface.

Crate naming: `libiot-<vendor>-<device>` for libraries,
`libiot-<vendor>-<device>-cli` for CLIs.

## Non-negotiables

- **Every commit passes `cargo fmt && cargo clippy --tests && cargo test`.**
  Run `/pre-commit` before committing — it wraps this plus doc checks and
  the workspace-wide keyword check.
- **Tests are required for ALL changes — especially bugfixes.** Every
  code change must ship with tests that exercise the new or modified
  behavior. Every bugfix must ship with a test that would have caught
  the bug. Untested code is incomplete code. This is non-negotiable and
  applies just as forcefully to one-line fixes as it does to new
  features. If existing coverage is insufficient for the area you're
  changing, add tests for the existing behavior *first*, then change it.
- **Rust 2024 edition, `thiserror` for errors, `tokio` for async, no
  `unsafe` in library code, no `unwrap`/`expect` in library code.**

## Conventions live in skills, not here

Detailed per-crate conventions — architecture layering, module and file
organization, constructor conventions, error-type patterns, import
style, rustdoc and doctest expectations, test file layout, authoritative
spec reference conventions, keyword enforcement, and more — are encoded
in the **`iot-crate-standards`** skill. That skill is triggered whenever
code is being added, modified, or reviewed in this repo. Read it before
touching any crate.

Project tracking conventions (every crate has a `project-tracker.md`,
how to update them, how to sync code TODOs) live in the
**`update-project-trackers`** skill.

## Workflow commands

- `/pre-commit` — run fmt, clippy, tests, doc checks before committing
- `/github-pr-autosubscribe` — subscribe to PR review activity after opening a PR
- `/review-iot-protocol` — review wire-format / codec / transport changes
- `/review-rust-api` — review public-API, error-type, and async-plumbing changes

The two `/review-*` commands are thin wrappers that dispatch to the
`iot-protocol-reviewer` and `rust-api-reviewer` agents respectively.
The agents run in isolated contexts so raw `sl diff` / Read output
from the review doesn't pollute the main session — only the findings
summary comes back.

### Review cycle

After roughly every 3 commits, and before marking a PR ready, run the
review command that matches the changes. For a brand-new crate, run
both. Fix findings in a follow-up commit before continuing.

## Markdown style

- **Tables**: pad every cell so that all cells in the same column have
  the same rendered width. This dramatically improves readability in
  the raw markdown (where most of us actually read it). When you add
  or update a row that's wider than the existing rows, re-pad the
  whole table. Applies to CLAUDE.md, READMEs, project trackers,
  skills, commands, PR descriptions — every `.md` file in this repo.

## Session planning docs (optional)

For plan-driven sessions, you may offer to save the planning document as
a date-prefixed `.md` file under `docs/`. **Always ask the user first** —
not every session needs one. Format: `docs/YYYY-MM-DD.TOPIC-PLAN.md`,
updated as you go, linked from the associated PR summary.

## Guidance capture

When the user provides direction that represents a reusable convention,
ask whether to encode it into CLAUDE.md or into a skill/command. Prefer
skills/commands over CLAUDE.md bloat — CLAUDE.md is loaded into every
session's context, so keeping it short is a first-class goal.
