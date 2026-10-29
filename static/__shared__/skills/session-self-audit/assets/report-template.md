# Session self-audit: <session-id>

Report path: <local path you wrote this to>
Written: <ISO timestamp>

This is a local report. It is not filed anywhere, posted anywhere, or
packaged for sharing — it is a markdown file on this machine for the user
to read.

## 1. Problem statement (REQUIRED)

<One paragraph: the session(s) in scope, the turn range if known, what was
expected, what happened, and the observable that matters — wall-clock,
tokens, repeated actions, one specific unexpected action.>

## 2. Audit verdict (REQUIRED)

<What the evidence shows happened around the reported problem. Prose, with
`path:line` after every claim. State confidence: high / medium / low, and
what would raise it.>

## 3. Environment (REQUIRED)

- OS:
- Harness and version:
- Models seen:
- Skill install root:
- Skill files read or injected (which, and whether their content was
  captured or only referenced):
- Other plugins, extensions, MCP servers:
- Instruction files present (paths only):

Label every environment field and skill observation as historical
evidence, unverified snapshot, current observation, or unknown, and record
its supporting evidence location.

## 4. Sessions examined (REQUIRED)

| Role | Session id | Absolute path | Lines | Bytes |
|---|---|---|---|---|

Rejected candidates: <id — path — why>, or "none".

## 5. Timeline (REQUIRED)

One row per human-typed prompt. Events column lists skills invoked,
subagents dispatched, compaction, errors, resumes, aborts.

| Turn | Line | Time | Request (one line) | Events |
|---|---|---|---|---|

## 6. Findings (REQUIRED, one subsection per dimension)

Each finding:
```
- finding: <one sentence>
  evidence: <path:line> — "<short quote>"
  turns: <first>–<last>
  confidence: high | medium | low
```
A dimension with nothing to report says `none found — checked: <what was
checked>`.

### 6.1 Skill timeline
### 6.2 Plan adherence
### 6.3 Repeated work
### 6.4 Stumbles
### 6.5 Quality evidence
### 6.6 Request conflicts
### 6.7 Cost and time
### 6.8 Other plugins and tools used

## 7. Coverage notes (REQUIRED)

- Not read: <ranges, files, and why>
- Harness features unavailable: <list or none>
- Session was in progress at read time: yes/no
- For the user to double-check: <list or none>
