---
name: auditing-skill-libraries
description: >-
  Use when a skill library needs a health check — after adding, renaming, splitting or
  deleting a skill, before a release that ships skills, when a skill stops being found or
  stops being obeyed, or when cross-references and reference files have drifted. Covers the
  bundled audit script, the mechanical invariants it enforces (byte cap, frontmatter, naming,
  link resolution, reference layout), how to read its findings, and search-gap mining for
  skills that ought to exist and do not.
---

# Auditing Skill Libraries

A skill library degrades invisibly. Nothing errors when a SKILL.md crosses the compaction cap,
when `name` stops matching its directory, when a reference link rots, or when a 400-line
reference has no table of contents. The skill still loads. It just stops working partway
through, in a session nobody is watching.

Audit mechanically, on a schedule and before release. Reading the library by eye finds the
prose problems and misses every one of these.

**REQUIRED BACKGROUND:** `writing-skills` defines the invariants this skill enforces. You
cannot fix a finding here without it.

## Run it

```bash
bash scripts/audit-skills.sh                      # infers the library it lives in
bash scripts/audit-skills.sh ~/.claude/skills     # audit an explicit library
bash scripts/audit-skills.sh --help

# An installed library also holds third-party skills pinned to upstream commits.
# Findings against those are noise — an update overwrites any fix. Exclude them:
bash scripts/audit-skills.sh ~/.claude/skills --exclude 'docker-*' --exclude 'n8n-*'
```

## Audit the library you maintain

Every check here assumes you can fix what it reports. That holds for skills you
author and not for skills vendored from an upstream commit, where the next update
replaces your fix. Run the audit over your own library, and exclude the vendored
ones by glob.

A run that reports hundreds of findings you cannot act on trains you to ignore the
tool, which costs more than the findings were worth. The excluded count is printed
so exclusion stays visible rather than silently shrinking the audit.

With no argument the script walks up from its own location to the library that contains it,
and falls back to `~/.claude/skills`. Pass a path when auditing a library it is not part of.

Exit code 0 means no ERROR. Exit code 1 means at least one invariant is broken — wire that
into whatever runs before a release. Exit code 2 means no library was found.

## What it checks

| Check | Severity | Why it matters |
|---|---|---|
| `SKILL.md` under 20,000 bytes | ERROR | After a compaction the file is re-attached truncated to 20,000 characters; the tail stops existing for the rest of the session |
| Frontmatter present on line 1 | ERROR | Without it the skill carries no metadata and is never selected |
| `name` and `description` present | ERROR | Same — these two fields are the entire selection surface |
| `name` equals the directory name | ERROR | A mismatch breaks invocation by name while leaving the file perfectly readable |
| Directory is kebab-case | ERROR | Mixed case and underscores make a skill unfindable by the name agents guess |
| No empty skill directories | ERROR | An empty directory reads as a skill that exists and does nothing |
| Markdown links resolve on disk | ERROR | A dead reference link is content the agent is told exists and cannot read |
| Reference over 100 lines has a table of contents | ERROR | Agents preview long files with partial reads and never learn what the rest holds |
| References exactly one level deep | ERROR | Agents follow one hop and stop, so a reference linked from a reference is read partially or not at all |
| Description within the 1024-character spec limit | ERROR | Over the limit the frontmatter is invalid |
| Description length under ~700 characters | WARN | Descriptions load into every conversation, whether the skill is used or not |
| Description does not read as a procedure | WARN | A step summary in the description becomes the version agents follow instead of the body |

Errors are invariants. Warnings are judgment calls — read the description before acting on
either description warning, because both are heuristics on prose.

## Reading the findings

| Finding | Fix |
|---|---|
| over the byte cap | Move the sometimes-needed sections into `references/` and link them from SKILL.md. Do not compress prose to squeeze under the cap; the content that only matters sometimes is the content to move. |
| `declares name: X — must equal the directory name` | Change whichever is wrong. If the directory is renamed, grep the library for the old name first. |
| `is not kebab-case` | Rename the directory, then grep for references to the old name. |
| `link target does not exist` | Either the file was never written or it moved. Create it or fix the link — never delete the link and leave the prose referring to it. |
| `is N lines with no table of contents` | Add a `## Contents` list of the file's own section headings at the top. |
| `links to X — references are linked from SKILL.md only` | Name the sibling file in prose instead of linking it, and link both from SKILL.md. |
| `description reads as a procedure` | Rewrite as triggers plus scope. Nouns the skill covers, never verbs in order. |
| `is an empty directory` | Delete it, or write the SKILL.md that was intended. |

## Search-gap mining

The audit finds broken skills. It cannot find missing ones — and a missing skill produces no
error at all, just an agent solving the same problem from scratch every time.

Log the searches. Every time a skill search runs, append the query and whether anything
matched to a line-delimited log beside the library:

```jsonl
{"ts":"2026-10-29T13:04:11Z","query":"flaky test timing","matched":0}
{"ts":"2026-10-29T13:19:02Z","query":"rotate leaked credential","matched":0}
```

Then tally the misses:

```bash
grep '"matched":0' ~/.claude/skills/.search-log.jsonl |
    sed -n 's/.*"query":"\([^"]*\)".*/\1/p' |
    sort | uniq -c | sort -rn | head -20
```

A query that returns nothing repeatedly is a skill request stated in the exact words the next
agent will search with. High count plus zero matches names the gap; use the query itself as the
seed for the description's trigger list, because it is already proven search vocabulary.

Two cheaper signals worth the same treatment:

- A query that matches but the agent reads and abandons — the description over-promised.
- The same technique explained from scratch in several transcripts — a skill that should exist.

## Extending the auditor

When you catch a recurring mechanical mistake, add a check to `scripts/audit-skills.sh` rather
than a paragraph to a skill. Prose asks an agent to remember; a check fails the run. Every rule
expressible as a pattern and moved out of prose is budget returned to the judgment calls.

Two traps the script is written to avoid, both of which produced silently useless audits
upstream:

- **Counters inside a pipeline are lost.** `find ... | while read; do n=$((n+1)); done` runs the
  loop body in a subshell, so `n` is zero afterwards and the summary reports a clean library
  no matter how many problems were printed. Use `while read ... done < <(find ...)` or a glob
  loop so the counter lives in the calling shell.
- **A default path pinned to one machine.** `SKILLS_DIR="${1:-$HOME/Documents/GitHub/...}"`
  passes on the author's laptop and audits an empty directory everywhere else — which also
  reports clean. Derive the default from the script's own location.

Both failures look identical from the outside: a green run. Prefer a check that fails loudly
when it cannot do its job.

## Common rationalizations

| Excuse | Reality |
|---|---|
| "I'll check the links by eye" | Dead links are found by the agent that needed the file, mid-task, with no fallback. The script takes a second. |
| "It's only just over the byte cap" | The truncation is not graceful. The tail stops existing from the first compaction onward. |
| "The name mismatch is cosmetic" | The directory is how the skill is invoked and the frontmatter is how it is identified. One of them is lying. |
| "That reference is short enough to read without a TOC" | 100 lines is the threshold because partial previews stop there. Count the lines rather than estimating. |
| "A warning isn't a problem" | A warning is a judgment call that has not been made yet. Make it and move on, rather than leaving it to re-surface every run. |
| "No search log, so no gap analysis" | Start the log now. The value is cumulative and today's missing skill is next quarter's repeated explanation. |
| "I'll audit before the next release" | Drift compounds, and a release is the worst moment to discover twenty findings. |

## Attribution

The library-health concept and the search-gap-mining idea are adapted from the
`gardening-skills-wiki` skill in [obra/superpowers](https://github.com/obra/superpowers)
(MIT, © 2025 Jesse Vincent). The audit scripts here are written from scratch; the
upstream ones are not portable and miscount.
`scripts/audit-skills.sh` is written from scratch — the upstream scripts count findings inside
a pipeline subshell and default to a hardcoded path, so they report a clean library in both
failure modes.

The invariants themselves — the compaction cap, one-level-deep references, and the 100-line
table-of-contents rule — come from Anthropic's published skill-authoring guidance.
