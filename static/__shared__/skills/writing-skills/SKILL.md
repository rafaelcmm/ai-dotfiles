---
name: writing-skills
description: >-
  Use when creating a new skill, editing an existing one, writing or fixing a skill's
  frontmatter description, splitting a SKILL.md that has grown too large, or deciding what
  form a piece of agent guidance should take. Covers skill and reference-file structure, the
  description field, the 20,000-byte SKILL.md cap, degrees of freedom, and the choice between
  prohibition, positive recipe, template slot, and conditional. Trigger it on "write a skill",
  "add a skill", "this skill is too long", "agents ignore this skill", "the skill never
  fires", or guidance that reads correctly and is still not obeyed under pressure.
---

# Writing Skills

A skill is a reference an agent loads mid-task to do something better than it would unaided.
It is not a narrative of how a problem was solved once, and it is not where project
conventions live — those belong in the instruction files.

The hard part is not deciding what to say. It is deciding what **form** to say it in. The
same content, phrased as a prohibition instead of a recipe, measurably produced worse output
than giving no guidance at all. Start there.

## Match the form to the failure

Classify the baseline failure first, then pick the form. The form that hardens one failure
type backfires on another.

| Baseline failure | Right form | Wrong form |
|---|---|---|
| **Discipline** — knows the rule, skips it under pressure | Prohibition + rationalization table + red flags ([bulletproofing](references/bulletproofing.md)) | Soft guidance: "prefer…", "consider…" |
| **Shaping** — complies, but the output has the wrong length, tone, or ordering | Positive recipe: state what the output **is** — its parts, in order | Prohibition list: "don't restate", "never narrate" |
| **Omission** — leaves out an element of something it already produces | A REQUIRED slot in the template it fills in | Prose reminders near the template |
| **Conditional** — behaviour should depend on a circumstance | A conditional on an observable predicate: "if the brief exists, cite it" | Unconditional rule plus exemption clauses |

### The measurement

Head-to-head wording tests on dispatch-prompt guidance, 5 reps per variant, fresh context per
rep. Score counts spec values needlessly re-typed into the composed prompt; lower is better.

| Variant | Score |
|---|---|
| Positive recipe — "your dispatch contains: (1)…(5)" | **3.0**, zero variance |
| No guidance at all (control) | 3.6 |
| Prohibition — "don't restate the brief" | **4.4** |
| Recipe plus one nuance clause | 3.8, noisy |

The distributions were fully separated. The prohibition arm was not noisily worse than the
recipe — it was consistently worse, and worse than saying nothing. Under a competing incentive
(restating the spec feels like helpful curation) an agent negotiates with "don't X". A recipe
leaves nothing to negotiate: the output either matches the stated shape or it does not.

### Prohibitions that do hold

Not every negative instruction backfires. These survived the same test series:

- **Tripwires** — a self-check on a concrete token: "if the prompt you are writing contains
  the words 'do not flag', stop and rewrite it." Fires reliably.
- **Recognition tables** — red-flag and rationalization tables, read at decision time rather
  than at composition time.
- **Discrete directives with no competing incentive** — "do not ask the reviewer to re-run
  the tests" measured 0/5 violations against a 3/5 control.
- **Policy gates** — "never push without permission" is policy, not output shaping. Leave
  those exactly as they are.

What backfires is specifically a **composition prohibition**: telling an agent what to leave
out of an artifact it has its own agenda for.

**Ties go to the shorter phrasing.** A long session re-reads an invoked SKILL.md many times
over; prose length is a recurring cost, not a one-time one.

## No nuance clauses

Appending one nuance clause to the winning recipe above ("quote only the fragment you need")
moved it from 3.0 with zero variance to 3.8 and noisy. A qualifier reopens the negotiation
that the recipe had closed.

**Exemption clauses do not scope.** "This length limit does not apply to code blocks" still
suppressed code blocks. The exemption reads as more pressure in the direction of the rule, not
as a carve-out.

Where an exception is real, do not write it as a clause. Write it as its own conditional on an
observable predicate, in its own sentence.

## The description field triggers, it does not instruct

The description is the only part of a skill loaded into every conversation. Its job is to let
an agent decide whether to read the body. Two things belong in it: **when** the skill applies,
and **what** it covers. One thing must never: **how** it works.

A description that summarizes the procedure becomes a shortcut agents take instead of reading
the skill. Diagnosed case: a description mentioning code review "between tasks" produced runs
with one review, while the skill body specified two. Deleting the workflow summary from the
description fixed it. The body had never been the problem.

This does not contradict Anthropic's "say what it does and when to use it". A capability
statement — the nouns a skill covers — helps an agent select it. A step sequence — verbs in
order — replaces it. Name the territory, never the route.

```yaml
# Bad — a procedure. Agents follow this and skip the body.
description: Use when executing plans — dispatch a subagent per task, review, then commit.

# Bad — too thin to select on.
description: For async testing.

# Good — triggers plus scope, no ordering.
description: >-
  Use when tests pass and fail inconsistently, hang, or depend on wall-clock timing. Covers
  condition polling, deterministic waits, and the failure modes of sleep-based tests.
```

Write it in the third person; it is injected into a system prompt. Load it with the words
someone would actually search for: error strings, symptoms, tool names, synonyms.

**When a near-miss trigger would load the skill wrongly, say what it is not for.** One
sentence, at the end of the description: "Not for a broken test or wrong output — that is a
debugging problem, not a stuck-on-approach problem." It costs a line and blocks a whole class
of wrong selection, which otherwise shows up as an agent reading the skill and abandoning it.

## Hard limit: SKILL.md under 20,000 bytes

After a compaction, Claude Code re-attaches an invoked skill **truncated to 20,000
characters**. Everything past the cut is gone for the rest of the session unless the agent
re-reads the file from disk, which nothing prompts it to do. A skill whose rules live in its
last 3,000 bytes is a skill whose rules silently stop existing mid-session.

Measure in bytes — stricter than characters, and typographic punctuation costs three each:

```bash
wc -c static/__shared__/skills/*/SKILL.md
```

Over the cap, or near it: move the content the skill needs only *sometimes* into
`references/`. The agent reads a reference fresh, from disk, at the step that needs it — which
is also cheaper. What stays in SKILL.md is whatever the agent needs in order to know that the
reference exists and when to open it.

## Degrees of freedom: match specificity to fragility

Anthropic's framing — think of the agent as a robot on a path:

- **Narrow bridge with cliffs on both sides.** One safe way forward. Give the exact command
  and say not to vary it. Migrations, release sequences, anything where a plausible-looking
  variation corrupts state.
- **Open field with no hazards.** Many routes succeed. Give direction and let the agent pick.
  Code review, naming, diagnosis.

| Freedom | Form | Use when |
|---|---|---|
| High | Prose principles and heuristics | Several approaches are valid; context decides |
| Medium | A parameterized template or pseudocode | A preferred pattern exists; variation is acceptable |
| Low | An exact command, few or no parameters | The operation is fragile, order matters, consistency is the point |

Over-specifying an open field burns tokens and leaves the skill stale the moment the codebase
moves. Under-specifying a narrow bridge is how an agent invents a flag that destroys data.

## The context window is a public good

A loaded SKILL.md competes with conversation history, every other skill's metadata, and the
user's actual request. Assume the agent is already capable; add only what it does not have.

Challenge every paragraph with one question: **does this justify its token cost?** Cut the
explanations of what a PDF is, what a library is, why testing matters. Keep whatever is
specific, surprising, or carries a failure mode.

A claim with no failure mode attached is usually filler. "Use meaningful names" is filler.
"Single-letter names defeat grep, which is how agents find call sites" is not.

## Reference files

- **One level deep.** Every reference links directly from SKILL.md. A reference that links to
  another reference gets partially read: agents follow the first hop and stop.
- **Over 100 lines needs a table of contents** at the top. Agents preview long files with
  partial reads, and without a TOC they never learn what the rest of the file holds.
- One subject per reference, named for that subject — `references/micro-testing.md`, never
  `references/notes.md`.

All three have the same failure mode: content the agent cannot see is content the agent does
not have.

## Cross-references: hard and soft

Two markers, and the difference is load-bearing:

- `**REQUIRED BACKGROUND:**` — a genuine prerequisite. The reader cannot apply this skill
  without the other one.
- `Related:` — a suggestion. Useful, optional.

Use the first sparingly and mean it. When every cross-reference is soft, a real prerequisite
reads as optional and gets skipped.

Never reference a file with `@path` syntax. It force-loads the file immediately, spending the
context before the agent needs it — the exact opposite of why the content was split out.

## Bundled scripts: name the interpreter

In prose, always invoke a bundled script through its interpreter:

```bash
bash scripts/audit-skills.sh      # correct
node scripts/render-graphs.js     # correct
```

Never a bare `scripts/audit-skills.sh`. Some packagers strip the executable bit in transit,
and the bare form then fails with `Permission denied` on a machine that is otherwise fine.

## Do not document what a validator can enforce

Mechanical constraints — byte caps, frontmatter presence, kebab-case directories, `name`
matching its directory — belong in a test or a hook, not in prose. Prose asks an agent to
remember. A check fails the build.

Documentation is for judgment calls. Every rule you can express as a regex and move out of the
prose is budget returned to the judgment calls.

This repo enforces its skill invariants in `src/tests.rs` (`cargo test`) and audits the whole
library through the `auditing-skill-libraries` skill.

## Persuasion principles — suggestive, not settled

Discipline skills read better when they borrow the structure of persuasive writing. The
evidence base is thin, so treat the table as a hypothesis to micro-test, not as a result.

| Principle | In a skill | Verdict |
|---|---|---|
| Authority | "YOU MUST", "No exceptions" — imperative, non-negotiable | Use for discipline skills |
| Commitment | Force an explicit choice; track checklist items as todos | Use for multi-step compliance |
| Scarcity | Bind to a moment: "before proceeding", "immediately after X" | Use against "I'll do it later" |
| Social proof | State the universal: "checklists without tracking get skipped — every time" | Use to establish a norm |
| Reciprocity | — | Avoid; the others do the work with less framing |
| Liking | — | **Never.** It trades honest technical judgment for agreeableness, manufacturing exactly the sycophancy that review skills exist to prevent |

**Required caveat before you use any of this.** The headline figure — compliance rising from
33% to 72%, N≈28,000 — comes from Meincke et al. 2025, which measured persuading models into
*objectionable* requests: a jailbreak-style setting. That those techniques can push a model
past its refusal behaviour does not establish that they improve adherence to legitimate
engineering discipline. The transfer is plausible, not demonstrated. Cite it as suggestive and
verify the wording on your own case.

Sources: Cialdini, R. B. (2021), *Influence: The Psychology of Persuasion*, Harper Business;
Meincke, L., Shapiro, D., Duckworth, A. L., Mollick, E., Mollick, L., & Cialdini, R. (2025),
*Call Me A Jerk: Persuading AI to Comply with Objectionable Requests*, University of
Pennsylvania.

## Verify the wording before you ship it

Reading a skill tells you it is clear to its author. It tells you nothing about whether it
binds another agent. Before shipping guidance whose whole purpose is to change behaviour, run
the protocol in [references/micro-testing.md](references/micro-testing.md): fresh-context
samples per variant, a no-guidance control, and every flagged match read by eye.

The control is the step people skip and the one that decides whether to write anything at all.
If the no-guidance control does not exhibit the failure, there is nothing to fix.

When the failure is a discipline failure, read
[references/bulletproofing.md](references/bulletproofing.md) before writing: closing
loopholes, the spirit-versus-letter line, and how to build the rationalization table.

## Common rationalizations

| Excuse | Reality |
|---|---|
| "It's obviously clear" | Clear to the author, who holds the whole context. The reader has one paragraph and a competing incentive. |
| "It's just a reference, nothing to test" | Reference skills fail at retrieval. Check that an agent can find the right section, then stop. |
| "A prohibition is more forceful" | On output shaping it measured worse than silence. Forceful and effective are different properties. |
| "I'll add 'unless it matters' to keep it flexible" | That clause turned a zero-variance recipe into a noisy one. Write the exception as its own conditional. |
| "The description should explain what the skill does" | Scope, yes. Steps, no — a step list in the description is the version agents actually follow. |
| "It's only slightly over 20,000 bytes" | The cut is not graceful. The tail stops existing after the first compaction. |
| "I'll document the byte cap inside the skill" | Document judgment, enforce mechanics. Byte caps belong in a test. |
| "One more example makes it clearer" | One excellent example beats five mediocre ones, and the same pattern in five languages is five things to maintain. |
| "I'll test it if problems show up" | A problem showing up means agents were quietly misapplying it for weeks. |

## Before you ship

Each line is a slot, not a suggestion. An unchecked box is an unfinished skill.

- [ ] Baseline failure classified, and the form matches it
- [ ] `name` is kebab-case and identical to the directory name
- [ ] `description` gives triggers and scope, third person, no step sequence
- [ ] If a near-miss trigger would load the skill wrongly: the description says what it is not for
- [ ] `wc -c SKILL.md` is under 20,000
- [ ] No nuance clauses and no exemption clauses anywhere in the file
- [ ] References one level deep; every reference over 100 lines opens with a TOC
- [ ] Bundled scripts invoked through their interpreter in the prose
- [ ] Mechanical rules moved into a test or hook instead of restated in prose
- [ ] One excellent example, not one per language
- [ ] If the skill changes behaviour: wording micro-tested against a no-guidance control
- [ ] If the skill is pure reference: an agent can locate the right section from SKILL.md alone
- [ ] Library audit clean — see `auditing-skill-libraries`

## Attribution

The form-to-failure mapping, the wording measurements, the nuance-clause finding, the
description-as-shortcut diagnosis, the bulletproofing toolkit and the persuasion notes are
adapted from [obra/superpowers](https://github.com/obra/superpowers) (MIT, © 2025 Jesse
Vincent). The guidance on degrees of freedom, context as a public good, tables of
contents over 100 lines, and one-level-deep references is Anthropic's own published
agent-skills documentation, which that repo vendors; it is credited to Anthropic, not to
the adaptation.

Degrees of freedom, the context-window-as-public-good framing, the 100-line table-of-contents
rule and one-level-deep references are Anthropic's published skill-authoring guidance, which
that project vendors.
