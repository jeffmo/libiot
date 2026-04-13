# libiot — Project Tracker

## Status: v0.1.0 — Initial implementation

## Completed

- [x] Crate scaffolding (Cargo.toml, bin config)
- [x] Error module: 20 variants, unique exit codes 10-29, kind() categories
- [x] Output module: OutputFormat, OutputContext, view structs, render functions
- [x] Settings module: load/save with atomic writes, 0600 perms, LIBIOT_CONFIG_DIR override
- [x] Discovery module: PATH scanning for libiot-* binaries, executable bit check
- [x] Clap argument parsing: all subcommand types, is_builtin()
- [x] Pre-parse dispatch: two-mode main(), delegation hint on misplaced flags
- [x] Commands: get/set/unset alias, get/set/unset env-var
- [x] Commands: list, list aliases, list env-vars
- [x] Commands: install (cargo subprocess, dry-run, --alias), uninstall (cleanup flags)
- [x] Commands: completions (dynamic subcommand augmentation)
- [x] Commands: config-path
- [x] Delegation: alias resolution, env var injection, Unix exec handoff
- [x] E2E tests (17), unit tests (87)
- [x] README.md

## Remaining / Future

- [ ] Windows support: spawn-and-wait fallback for non-Unix
- [ ] Dynamic shell completions: regenerate on install/uninstall hint
- [ ] Delegated sub-CLI completion forwarding
- [ ] `libiot update` command (cargo install --force all installed CLIs)
- [ ] `libiot doctor` command (verify all aliases point to valid binaries)

## Unresolved Questions

- Should `list env-vars` (no filter) show resolved/merged vars or raw settings?
  Currently shows raw settings groups. Resolved vars are shown only with a filter.
- Should `NoCLIsFound` be a warning rather than an error for `list`?
  Currently exit code 22 when no CLIs found on PATH.
