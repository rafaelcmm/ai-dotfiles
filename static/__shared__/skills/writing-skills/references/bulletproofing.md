# Bulletproofing a discipline skill

Part of `writing-skills`. The toolkit for closing loopholes in a rule an agent knows and skips
anyway.

## Contents

1. [Scope — read this first](#1-scope--read-this-first)
2. [Close named loopholes, not the general case](#2-close-named-loopholes-not-the-general-case)
3. [Cut off spirit-versus-letter early](#3-cut-off-spirit-versus-letter-early)
4. [Build the rationalization table from real transcripts](#4-build-the-rationalization-table-from-real-transcripts)
5. [Write a red-flags list](#5-write-a-red-flags-list)
6. [Put violation symptoms in the description](#6-put-violation-symptoms-in-the-description)
7. [Done is measured, not felt](#7-done-is-measured-not-felt)
8. [Attribution](#8-attribution)

---

## 1. Scope — read this first

**This applies to one failure type only: a discipline failure.** The agent understands the
rule, can state it correctly, and violates it under pressure — time, sunk cost, authority,
exhaustion, or a plausible-sounding "this case is different".

Do not apply any of it to the other failure types:

| Failure | Use instead |
|---|---|
| Output has the wrong length, tone, or ordering | A positive recipe stating what the output is |
| A required element is missing from an artifact | A REQUIRED slot in the template |
| Behaviour should depend on a circumstance | A conditional on an observable predicate |

On output shaping this toolkit does not merely underperform — a prohibition arm measured worse
than giving no guidance at all. Classify the failure before reaching for a prohibition.

## 2. Close named loopholes, not the general case

A bare rule invites a search for the gap. Name the gaps.

Weak:

```markdown
Found a secret in a committed file? Rotate it.
```

Bulletproof:

```markdown
Found a secret in a committed file? Treat it as leaked and rotate it.

**No exceptions:**
- Not when the repository is private
- Not when the commit was never pushed
- Not when it is "only" a staging credential
- Not when you plan to rewrite the history afterwards
```

Each bullet exists because an agent used it. Loopholes you have not observed are speculation;
loopholes captured from a baseline run are data. Add a bullet when a rep finds the gap, not in
anticipation of one.

## 3. Cut off spirit-versus-letter early

State the foundational principle near the top of the skill, ahead of any detail:

```markdown
**Violating the letter of this rule is violating its spirit.**
```

That one line retires an entire class of rationalization — "I'm honouring the intent
differently", "the purpose is X and I achieved X another way". Without it, every specific
counter you write gets re-litigated at a higher level of abstraction.

## 4. Build the rationalization table from real transcripts

Two columns. The left column is **verbatim**. Paraphrasing is what breaks the mechanism: an
agent recognizes its own sentence and stops, but it does not recognize a tidied-up version of
it.

Example, for a skill requiring that a claim of completion be backed by a command that was
actually run:

```markdown
| Excuse | Reality |
|---|---|
| "The change is too small to re-run the suite" | Small changes break builds; the suite takes less time than the follow-up bug report does. |
| "I read the diff and it's obviously correct" | Reading proves you understood your intent, not that the code does it. |
| "The user is waiting, I'll verify after reporting" | A report of success is what they act on. Verify first or report it as unverified. |
| "CI will catch it" | CI catches it after the claim is already on record as done. |
```

Each row answers the excuse on its own terms. "That's wrong" closes nothing; a consequence the
agent had not weighed does.

Table hygiene:

- One row per distinct rationalization, no merging
- Reality column states a consequence, not a restatement of the rule
- No row for an excuse nobody actually made

## 5. Write a red-flags list

A rationalization table is read when an agent is already arguing. A red-flags list is read
just before it starts — the list is a self-check on the agent's own phrasing.

```markdown
## Red flags — stop

- You are about to write "this case is different because…"
- You are about to report success from reading the code rather than running it
- You caught yourself weighing whether the rule applies at all
- You are reaching for the word "pragmatic"

**All of these mean: follow the rule as written.**
```

Phrase each flag as a concrete token or phrase the agent would actually emit. Abstract flags
("you are rationalizing") never fire. Token-level flags fire reliably, and that reliability is
the one thing prohibition-shaped guidance measurably does well.

## 6. Put violation symptoms in the description

The description decides whether the skill loads at all, so it has to trigger on the moment
*before* the violation, not on the clean case.

```yaml
# Weak — only fires when things are already going well
description: Use when verifying a change.

# Bulletproof — fires at the moment of temptation
description: >-
  Use when about to report work as done, when a change looks too small to re-check, when the
  suite is slow and skipping it is tempting, or when confidence comes from reading the diff
  rather than running anything.
```

Still no procedure in the description. Symptoms and triggers only.

## 7. Done is measured, not felt

A discipline skill is finished when, under maximum pressure, an agent chooses correctly, cites
the section that made it choose, and acknowledges the temptation it declined. It is not
finished while an agent still invents fresh rationalizations, argues the rule is wrong, or
proposes a hybrid that technically satisfies the words.

Re-run the scenario after every edit. A counter added for one loophole regularly opens
another, and another rep is the only way to find out. The protocol, sampling and scoring rules
are in `micro-testing.md`, the sibling reference.

## 8. Attribution

Adapted from the bulletproofing and skill-testing material in
[obra/superpowers](https://github.com/obra/superpowers) (MIT, © 2025 Jesse Vincent).
