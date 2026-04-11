Run a wire-format / codec / transport review of recently-changed code
in this workspace.

Use the `iot-protocol-reviewer` agent via the Agent tool. Pass it the
following prompt:

> Review the wire-format, codec, and transport changes in this
> workspace. Run `sl diff` (or `git diff HEAD~3..HEAD` if no working
> copy changes) to find what changed. Walk through your standard
> review domains (wire-format conformance, parser robustness,
> unsolicited frames, broadcast semantics, metadata trailers, test
> coverage of the wire format) and produce a structured findings
> report.

If the user specified particular files or a particular PR range, pass
those along to the agent instead of using the default `sl diff` scope.

After the agent returns its findings, summarize them in the main
session (listing critical / warning / note counts and the headline
issues) and ask the user whether to proceed with fixes.
