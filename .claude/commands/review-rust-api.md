Run a public-API / error-type / async-plumbing review of
recently-changed code in this workspace.

Use the `rust-api-reviewer` agent via the Agent tool. Pass it the
following prompt:

> Review the public-API, error-type, and async-plumbing changes in
> this workspace. Run `sl diff` (or `git diff HEAD~3..HEAD` if no
> working copy changes) to find what changed. Walk through your
> standard review domains (Rust API Guidelines, error hygiene, async
> correctness, type-driven design, edition 2024 idioms, documentation,
> clippy pedantic cleanliness, import style, module/file organization,
> test organization) and produce a structured findings report.

If the user specified particular files or a particular PR range, pass
those along to the agent instead of using the default `sl diff` scope.

After the agent returns its findings, summarize them in the main
session (listing critical / warning / note counts and the headline
issues) and ask the user whether to proceed with fixes.
