Run pre-commit verification to ensure code is ready for commit. Execute
these checks in order and report results:

1. **Format**:   `cargo +nightly fmt --all -- --check`
2. **Lint**:     `cargo clippy --workspace --all-targets --all-features -- -D warnings`
3. **Tests**:    `cargo test --workspace --all-features`
4. **Docs**:     `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features`
5. **Keywords**: `./scripts/check-libiot-keywords.sh`

After all checks:
- If all pass: report success and show `sl status`
- If any check fails: explain the failure clearly, provide the specific
  error message, suggest fixes, and do NOT attempt to commit

At minimum, the baseline every commit must pass is
`cargo fmt && cargo clippy --tests && cargo test`. Steps 4 and 5 catch
rustdoc rot and the libiot keyword regression respectively — don't skip
them unless you have a deliberate reason.

If the nightly rustfmt toolchain isn't installed, run
`rustup component add rustfmt --toolchain nightly` first. The workspace
`rustfmt.toml` uses several unstable rustfmt options (`imports_granularity`,
`group_imports`, `match_block_trailing_comma`) that only take effect on
nightly.
