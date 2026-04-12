# libiot-cli

Unified CLI dispatcher for the [libiot](../../README.md) ecosystem.

`libiot` discovers all installed `libiot-*` CLI binaries on `$PATH`
and exposes them as subcommands — similar to how `git` dispatches to
`git-*` programs. It also provides alias management, per-command
environment variable injection, and `cargo install`/`uninstall`
wrappers.

## Quick start

```bash
# Install the dispatcher
cargo install libiot-cli

# Install a device CLI
libiot install rollease-automate-pulse-pro-hub

# Use it via the full name
libiot rollease-automate-pulse-pro-hub hub info

# Or create a short alias
libiot set alias rollease-automate-pulse-pro-hub shades
libiot shades hub info

# Set a persistent env var so you don't have to pass --hub every time
libiot set env-var shades PULSE_PRO_HUB 192.168.1.2
libiot shades hub info   # LIBIOT_PULSE_PRO_HUB is injected automatically
```

## Two operating modes

**Built-in commands** — manage aliases, env vars, and installations:

| Command                                | Description                              |
|----------------------------------------|------------------------------------------|
| `libiot set alias CMD ALIAS [-f]`      | Create an alias for a CLI command        |
| `libiot unset alias ALIAS`             | Remove an alias                          |
| `libiot get alias ALIAS`               | Show what an alias points to             |
| `libiot set env-var CMD VAR VALUE`     | Set a per-command env var                |
| `libiot unset env-var CMD VAR`         | Remove a per-command env var             |
| `libiot get env-var CMD VAR`           | Show a per-command env var value         |
| `libiot list`                          | List all CLIs and aliases                |
| `libiot list aliases`                  | List all aliases                         |
| `libiot list env-vars [CMD]`           | List env vars, optionally for one cmd    |
| `libiot install NAME [--alias ALIAS]`  | Install a libiot CLI via cargo           |
| `libiot uninstall NAME`                | Uninstall a libiot CLI via cargo         |
| `libiot config-path`                   | Print the settings file path             |
| `libiot completions SHELL`             | Generate shell completions               |

**Delegation mode** — anything else is exec'd to the matching
`libiot-*` binary:

```bash
libiot rollease-automate-pulse-pro-hub hub info
# equivalent to: libiot-rollease-automate-pulse-pro-hub hub info
```

Top-level flags (`--format`, `--quiet`) are only available for built-in
commands. In delegation mode, all arguments are passed through verbatim.

## Environment variables

Env vars set via `libiot set env-var` are stored without the `LIBIOT_`
prefix. The prefix is added automatically at execution time. For
example:

```bash
libiot set env-var shades PULSE_PRO_HUB 192.168.1.2
# When running `libiot shades ...`, the child process sees:
#   LIBIOT_PULSE_PRO_HUB=192.168.1.2
```

When using an alias, env vars are resolved in order:
1. Underlying command's env vars (base)
2. Alias-specific env vars (override)

## Configuration

Settings are stored in `~/.config/libiot/settings.json` (override with
`LIBIOT_CONFIG_DIR`). The file is created on first write with 0600
permissions.

## Output formats

- `--format human` (default) — aligned, human-readable text
- `--format json` — structured JSON for scripting
- `--quiet` / `-q` — suppress non-error output
