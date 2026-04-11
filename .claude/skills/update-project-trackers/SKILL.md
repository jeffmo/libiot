---
name: update-project-trackers
description: Track, organize, and maintain project-tracker.md files and code TODOs for the libiot workspace. Use when the user asks to update project trackers, sync TODOs, mark tasks complete, add new tasks, identify high-impact work, or asks what's left to do in the libiot codebase. Triggers include phrases like "update project trackers", "sync TODOs", "what's left to do", "mark X as done", "track a new task", "highest-impact work", or references to project-tracker.md files.
---

# update-project-trackers

Manage project tracking for the `libiot` workspace via `project-tracker.md`
files and code TODO comments.

## Overview

The libiot workspace tracks work in two ways:

1. **`project-tracker.md` files** — Structured tracking documents at
   crate roots (one per crate) and optionally at the workspace root
   for cross-cutting workspace concerns.
2. **Code TODOs** — Inline `// TODO:` / `// FIXME:` / `// HACK:`
   comments marking work to be done.

This skill keeps those two views synchronized and helps prioritize
what to work on next.

## File hierarchy

```
libiot/
├── project-tracker.md                                                  # Optional workspace-level items
└── crates/
    ├── libiot-rollease-automate-pulse-pro-hub/project-tracker.md       # Per-crate tracker
    ├── libiot-samsung-frametv/project-tracker.md                       # (future)
    └── libiot-lutron-radiora2/project-tracker.md                       # (future)
```

**Rule:** TODOs go to the nearest `project-tracker.md` — if the file
lives in a crate, use that crate's tracker; otherwise use the
workspace-root tracker (creating it if it doesn't yet exist).

## Workflows

### 1. Sync TODOs from codebase

When asked to sync or update project trackers:

1. Run `.claude/skills/update-project-trackers/scripts/scan_todos.py <repo-path>`
   to find all TODOs.
2. For each `project-tracker.md` file, compare found TODOs against the
   "Appendix: Code TODOs" table.
3. **New TODOs:** Present them to the user for review before adding to
   the appropriate section of the tracker.
4. **Missing TODOs:** If a TODO in the appendix no longer exists in
   code, it may have been completed or removed — investigate and
   update accordingly.
5. Regenerate the "Appendix: Code TODOs" table.
6. Update the "Last Updated" date.

### 2. Mark work complete

When asked to mark something done:

1. Locate the item by section number (e.g., "2.1") or description.
2. **Wholly complete:**
   - Move to "Past Completed Work" section with a simple title, terse
     description, and date.
   - Check all "Definition of Done" boxes.
3. **Partially complete:**
   - Leave in place.
   - Update "Current Progress" to reflect what's done.
   - Update remaining tasks to reflect what's left.
4. **NEVER re-number identifiers** — IDs like 2.1, 4.3 must remain
   stable so external references keep working.
5. Update the "Last Updated" date.

### 3. Add a new task

When asked to track a new task:

1. Determine the appropriate `project-tracker.md` file based on which
   crate (or workspace-level concern) it affects.
2. Determine the appropriate section (or create a new section if
   needed).
3. Draft the new item following the format in
   `references/project_tracker_format.md`.
4. **Present to the user for review before adding.**
5. Assign the next available ID within that section (never reuse IDs).
6. Update the "Last Updated" date.

### 4. Identify high-impact work

When asked what to work on next:

1. First, sync TODOs and update all `project-tracker.md` files.
2. Analyze by priority markers (HIGH/MEDIUM/LOW) in the Priority
   Summary.
3. Consider dependencies (blocked items vs ready items).
4. Consider scope (quick wins vs large efforts).
5. Present top 3-5 recommendations with rationale.

## TODO comment patterns

Scan for these patterns in `.rs` files:

**Explicit markers:**

- `// TODO:` or `// TODO` — Standard TODO.
- `// FIXME:` or `// FIXME` — Bug or broken code.
- `// NOTE:` — May indicate a future consideration. Exclude these if
  they only explain something and don't indicate a need to come back
  and change or otherwise take action.
- `// HACK:` — Temporary solution needing cleanup.

**Semantic patterns** (use judgment):

- Comments mentioning "fix this", "clean up", "reconsider", "revisit"
- Comments about "temporary", "workaround", "should be changed"
- Comments with future tense about changes ("will need to", "should
  eventually")

Not every comment needs to become a tracked item — only clear action
items.

## project-tracker.md format

See `references/project_tracker_format.md` for the full template.

General style:

- All markdown table cells in a column should have consistent width
  for human-readability.
- Project trackers are deliberately terse — **sacrifice grammar for
  concision.**
- End each tracker with an "Unresolved Questions" section listing any
  open questions that need user input before the next chunk of work
  can be scoped properly.

Key sections:

- **Current State Summary** — Test counts, implementation status
- **Numbered Sections** — Grouped by category
- **Priority Summary** — HIGH/MEDIUM/LOW categorization
- **Past Completed Work** — Archive of finished items
- **Unresolved Questions** — Open questions awaiting user input
- **Appendix: Code TODOs** — Auto-generated table of inline TODOs

Each item includes:

- **Purpose** — Why this matters
- **Current Progress** — What's done
- **Priority** — HIGH/MEDIUM/LOW
- **Tasks** — Numbered subtasks
- **Definition of Done** — Checkboxes for completion criteria

## Important rules

1. **Stable IDs:** Never renumber items. If Section 2.1 is completed,
   the next item in Section 2 is 2.7 (or whatever follows), not 2.1.
2. **Always regenerate the Code TODOs appendix** when updating any
   `project-tracker.md`.
3. **Ask before adding:** New items from TODO scans should be
   presented for user review.
4. **Update timestamps:** Always update the "Last Updated" date when
   modifying a `project-tracker.md`.
5. **Terse completions:** When moving to "Past Completed Work", use
   only a title and one-line description.
