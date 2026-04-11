# project-tracker.md Format Reference

Template and conventions for `project-tracker.md` files in the libiot
workspace.

## Document structure

```markdown
# [crate-name] — Project Tracker

**Last Updated:** YYYY-MM-DD

Terse tracker — grammar sacrificed for concision. Managed via the
`update-project-trackers` skill.

## Document Maintenance Notes

When updating this document:

1. **Completed items:** Move wholly-completed items to the "Past
   Completed Work" section at the end of this document. Include a
   simple title and terse description only.
2. **Item identifiers:** NEVER re-number existing items (e.g., 4.3,
   2.1). This ensures references to IDs remain valid over time.
3. **Partial completion:** If an item is partially done, leave it in
   place and update its description to reflect remaining work.

---

## Current State Summary

**Test Status:** [X tests passing, Y doctests passing]

**Core Implementation: [STATUS]**
- [Brief bullet points of what's implemented]

**Remaining Work Categories:**
1. [Category Name] (Section 1)
2. [Category Name] (Section 2)
...

---

## Section 1: [Category Name]

### 1.1 [Item Title]

**Purpose:** [Why this matters]

**Current Progress:** [What's done so far]

**Priority:** HIGH | MEDIUM | LOW

[Optional: **Depends on:** Section X.Y]

#### Tasks

1. **[Task name]**
   - Detail
   - Detail

2. **[Task name]**
   - Detail

#### Code References (if applicable)
- `file.rs:123`: Brief note

### Definition of Done
- [ ] Criterion 1
- [ ] Criterion 2
- [ ] Criterion 3

---

## Section 2: [Next Category]

### 2.1 [Item Title]
...

---

## Priority Summary

**HIGH Priority:**
- [Item title] (Section X.Y) — [brief reason]

**MEDIUM Priority:**
- [Item title] (Section X.Y)

**LOW Priority:**
- [Item title] (Section X.Y)

---

## Past Completed Work

*Items moved here when wholly completed. Each entry is a simple title
and terse description.*

### [Item Title] (YYYY-MM-DD)
[One-line description of what was completed]

### [Item Title] (YYYY-MM-DD)
[One-line description]

---

## Unresolved Questions

*Open questions awaiting user input. Remove an entry once answered and
the answer is reflected in the relevant section above.*

- [ ] Does X need Y, or should we do Z instead?
- [ ] Is the priority on section 2.3 still correct given recent changes?

---

## Appendix: Code TODOs

TODOs found in the codebase (auto-generated — do not hand-edit):

| File | Line | TODO |
|------|------|------|
| `file.rs` | 123 | Brief description |
| `file.rs` | 456 | Brief description |
```

## Section numbering rules

- Sections are numbered 1, 2, 3, etc.
- Items within sections are numbered X.1, X.2, X.3, etc.
- **NEVER renumber** — if 2.3 is completed, the next new item is 2.7
  (or the next unused number). This ensures external references to
  "Section 2.3" remain valid.

## Priority levels

- **HIGH:** Blocks other work, is user-visible correctness, or closes
  a core functionality gap.
- **MEDIUM:** Important for completeness; enables downstream work.
- **LOW:** Nice-to-have, optimization, or polish.

## Completion workflow

When wholly completing an item:

1. Check all "Definition of Done" boxes.
2. Move the item to the "Past Completed Work" section.
3. Format: `### [Title] (YYYY-MM-DD)` + one-line description.
4. Update the "Last Updated" date at top.

When partially completing:

1. Update "Current Progress".
2. Revise remaining tasks.
3. Check completed "Definition of Done" boxes.
4. Update the "Last Updated" date at top.

## Code TODOs appendix

The appendix table should be regenerated whenever updating the
`project-tracker.md`. Format:

```markdown
| File | Line | TODO |
|------|------|------|
| `codec/parser.rs` | 87 | Handle unknown trailer bytes gracefully |
```

- File paths are relative to the crate root.
- Line numbers should be current (regenerate when updating).
- TODO text should be concise — truncate if needed.
