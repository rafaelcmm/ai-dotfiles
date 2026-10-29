---
name: commenting-standards
description: >-
  Language-agnostic commenting and documentation standards. Every public construct
  carries a docblock; every docblock is assertive, objective, and describes the code
  as it is now. Comments explain WHY, never narrate change history. Applies to any
  language or paradigm. Load before writing or reviewing code comments, docblocks,
  JSDoc/TSDoc, docstrings, rustdoc, or Go doc comments.
allowed-tools: [Read]
---

# Commenting Standards

A comment is a claim about the code it sits on. Its job is to make that claim
true, useful, and short — in that order.

Two obligations, both mandatory and independent:

1. **Coverage.** Every public construct has a docblock. Non-negotiable.
2. **Economy.** Every docblock earns each line it occupies.

Most documentation fails the second while satisfying the first. A docblock that
is thorough and padded is a worse artifact than a short one, because length
hides the part that mattered.

---

## Rule 1 — Document the present

**A comment describes what the code does now. It is not a record of how the code
got here.**

Version control owns history. `git log`, `git blame`, the PR, and the issue
tracker already hold it, with authorship and dates attached, and they stay
accurate for free. A comment that narrates change is worse than redundant: it
goes wrong the moment the next change lands, and it is the one kind of staleness
a reader cannot detect — nothing in the file contradicts it.

Never write:

| Banned | Why |
|---|---|
| `Extracted from UserService when billing needed it too` | Provenance. `git log --follow` answers this. |
| `This used to batch requests, but that resolver was removed` | Describes code that is not here. |
| `Until 2026-09-17 the opposite applied here` | A dated claim nobody will revisit. |
| `SUPERSEDED by rebalance.mjs on 2026-09-17` | Belongs in the superseding commit, or in a guard that throws. |
| `Renamed from parseInput; see plan 054 phase 3` | Plan archaeology. The plan is not the code. |
| `Previously we used a Map here, now it's a Set` | The diff already said this, once. |
| `Added for the checkout refactor` | Says nothing about current behaviour. |

Write the fact, not the journey:

```diff
-/**
- * Shipping quote cache.
- *
- * Originally this fanned out one request per item with Promise.all, which
- * opened 147 concurrent connections on a cold cache and got us throttled.
- * Plan 054 removed that resolver, so the fan-out is gone.
- *
- * Previously the TTL was 1 hour. Raised to 6 after we measured carrier table
- * churn in August; see the discussion on PR #812. Note that an earlier
- * revision keyed on the full postal code, which was changed to the district
- * prefix during the privacy review (plan 061 phase 2).
- *
- * TODO(2024): this used to live in lib/checkout before the shipping module
- * was extracted. Some callers may still import it from the old path.
- */
+/**
+ * Caches shipping quotes for 6 hours, keyed by postal district.
+ *
+ * Carrier price tables move on the order of months, so a 6-hour entry cannot
+ * serve a price a shopper would notice as wrong. The key is the district
+ * prefix, not the full postal code: that is the granularity carriers price
+ * at, and it keeps a full personal postal code out of the cache.
+ *
+ * Callers must not add a per-item upstream lookup here — this runs on a
+ * public endpoint, and one request per item will trip the carrier's rate limit.
+ */
```

Fifteen lines become eleven, and every surviving line is actionable. Three
facts a maintainer needs — the TTL and why it is safe, why the key is truncated,
and the per-item lookup that must never be added — were absent from the original
or buried in it. Everything deleted was a story about code that is no longer
there, including a stale `TODO` about an import path that moved two refactors
ago.

### What may legitimately carry time or context

These are present, actionable facts, not history:

- `TODO`/`FIXME` with a tracking reference — an open obligation.
- A workaround, with the upstream issue link and the condition that retires it.
- A deprecation marker naming the replacement.
- An invariant with its expiry trigger — "re-fit these buckets if the price
  distribution shifts materially".
- A measured number whose derivation a reader needs in order to re-derive it —
  state the method, not the expedition.

The test: **would a reader take a different action because this is here?** If
yes, keep it. If it only explains how the file came to look this way, delete it.

**This rule governs code comments, not dated documents.** A design document,
spec, plan, or ADR with a date on it is a point-in-time snapshot, and rewriting
it rewrites history. Code describes now; a dated document is allowed to be a
record.

---

## Rule 2 — What a docblock contains

**A docblock is one declarative sentence naming what the construct does or
guarantees, followed only by the facts a caller needs in order to use it
correctly: the constraints a type cannot express, what is guaranteed on
success, and how failure is signalled.**

That is the contents. Coverage is uniform; depth is not, and depth is set by
how far a mistake travels:

| Construct | What the docblock owes |
|---|---|
| Self-evident private helper | One line. Often the signature plus a name is already the contract. |
| Internal function, obvious behaviour | One sentence stating what it returns and any constraint a caller could get wrong. |
| Internal function, subtle behaviour | That, plus the invariant or edge case that makes it subtle. |
| Public API | Full contract: semantics of inputs, what is guaranteed on success, how failure is signalled. |
| Public API that is dangerous, lossy, or irreversible | The contract, plus the specific way callers get it wrong and the consequence. |

Three lines look like contract and are usually padding. Each earns its place
under one condition:

| Line | Earns its place when |
|---|---|
| A concurrency, ordering, idempotency, or lifecycle note | The construct behaves differently under concurrent use, out-of-order calls, repeated calls, or a lifecycle stage. On a pure function it is padding. |
| A `@returns` description | It carries what the return type does not state. One that restates the type is padding — and the padding is the empty *description*, never the tag itself; see "Economy applies to tag content" below. |
| A "role in the wider system" paragraph | The construct is reached from outside the module that owns it. On a two-line formatter it is padding. |

Padding is a defect, because length hides the line that mattered.

### The stop rule

A docblock is finished when **a competent caller can use the construct correctly
without reading its body**.

Everything past that point has a home elsewhere: implementation reasoning goes
in an inline comment next to the line it explains, design rationale goes in a
design doc or an ADR, and history goes nowhere.

---

## Rule 3 — How a docblock reads

**A docblock reads as a sequence of statements about the construct: indicative
mood, present tense, active voice, one idea per sentence.** "Returns the
normalized host." The first line is the one a hurried reader takes away, so it
names what the construct does or guarantees and is correct on its own.

The sentence shapes that produce this:

- **The construct is the subject, and the sentence opens on the verb.**
  "Converts a currency amount to micros." The reader can already see what the
  docblock is attached to, so "This function/method/class…" spends words
  without adding one.
- **A value's description carries what its type cannot**: what makes a value
  valid, what its bounds are, what happens at zero. `@param count number — the
  count` states nothing the signature did not.
- **Every adjective resolves to a consequence.** In place of "handles this
  carefully", name what breaks when a caller does not.
- **Two short sentences in place of one with a chain of subordinate clauses.**
- **Each word carries a constraint, a guarantee, or a failure.** The words that
  reliably carry none: *basically, essentially, simply, just, note that, it is
  important to, please be aware, in order to, there is a X which*.

Before — 14 lines, four facts:

```
/**
 * This is a utility function which is used in order to basically convert a
 * currency amount into the micros format.
 *
 * It is important to note that the Google Ads API will essentially always
 * expect monetary values to be passed in as micros, which are millionths of
 * a currency unit. This function should be used whenever you need to send
 * a monetary value.
 *
 * @param amount {number} The amount, as a number.
 * @returns {number} Returns the amount in micros as a number.
 */
```

After — 4 lines, the same four facts plus the one that actually matters:

```
/**
 * Converts a currency amount to the micros the API expects.
 *
 * Rounded, not truncated: the API rejects a non-integer, and truncation
 * under-bids by up to one micro on every call.
 */
```

The second is shorter because the padding left, not because information did.
**Brevity is the symptom of precision, never the goal.** Never delete a fact to
hit a line count.

### Economy applies to tag content, never to tag presence

Where a language's documentation convention expects structured tags or sections,
**keep them**. TSDoc and JSDoc `@param` / `@returns` / `@throws`, Python's
`Args:` / `Returns:` / `Raises:`, and rustdoc's `# Errors` / `# Panics` sections
are part of the contract and are consumed by tooling, editors, and generated API
references. Dropping a tag to shorten a docblock removes a machine-readable fact,
which is the opposite of this skill's intent.

TypeScript in particular reinforces full TSDoc: see
`typescript-documentation-best-practices` for the required tag set.

What changes is what goes *inside* the tag. Make the description carry
information the signature does not:

```diff
-@param amount {number} The amount, as a number.
-@returns {number} Returns the amount in micros as a number.
+@param amount - Major currency units. Fractional values are rounded, not truncated.
+@returns Integer micros, as the API requires.
```

Omit a tag only when the language's convention treats it as optional *and* it
would say nothing beyond the type — never merely to save a line.

---

## The two kinds of comment

### Docblocks — the contract

Document what stays true even when the implementation changes. Required on every
class, module, function, method, getter, setter, and public property.

Every docblock on a public construct carries these three, in this order:

- [ ] First line states what it does or guarantees, declaratively
- [ ] The behavioural guarantees a caller may rely on
- [ ] How failure reaches the caller — what it throws, returns, or silently drops

An internal construct carries what its row in Rule 2's proportionality table owes.

Then one line for each of these that holds:

- [ ] A parameter whose valid values the type does not pin → the constraint, the valid range, or the edge value
- [ ] A return whose empty, zero, or null case carries meaning → what that case means
- [ ] Behaviour that differs under concurrent use, out-of-order calls, repeated calls, or a lifecycle stage → the constraint
- [ ] A construct reached from outside the module that owns it → its role in the wider system

### Inline comments — the reasoning

Explain the non-obvious decision at the line it affects. Required when logic is
subtle, a domain rule drives the behaviour, an external system imposes a
constraint, or a side effect is invisible from the syntax.

- [ ] Explains why this approach, not what the line does
- [ ] States the invariant the surrounding code preserves
- [ ] Justifies a non-obvious branch or early return
- [ ] Carries a tracking reference for a workaround
- [ ] Absent on self-evident statements

> **Private members** may mix contract and implementation detail freely — a
> private member *is* an implementation detail.

---

## Anti-patterns

| Anti-pattern | Example | Why it fails |
|---|---|---|
| Change narration | `// Extracted from X when Y needed it` | History. Git owns it; the comment rots undetectably. |
| Dated claim | `// As of 2026-09-17 this is the opposite` | Nobody revisits it; it silently becomes a lie. |
| Changelog block | A docblock listing what each refactor altered | A file-local changelog that competes with the real one. |
| Name restatement | `// Saves the term` above `saveTerm()` | Zero information added. |
| Empty tag description | `@param id string — the id` | Keep the tag; the description must add what the type cannot. |
| Hedged contract | `// Should normally return the user` | A contract that commits to nothing is not a contract. |
| Padded trivial docblock | 12 lines on a two-line pure function | Buries the one line that mattered. |
| Vacuous header | `/** SearchBox view model */` | Omits role, responsibility, lifecycle. |
| Syntax narration | `// Increment i` above `i++` | Restates the obvious. |
| Undocumented invariant | Filter-then-reinsert with no comment | The rule being enforced is invisible. |

---

## Reviewing existing comments

When touching documented code, the comments are part of the change.

- A comment contradicted by the code is a **bug**. Fix it in the same change.
- History narration found in a comment you are already editing: **delete it**.
  Do not preserve it, do not append to it, do not convert it to a bullet list.
- Never append a new paragraph describing what this change did. Rewrite the
  claim so it describes the code as it now stands.
- A stale comment removed is a real improvement. It needs no replacement.

The failure mode to watch for: each change appends a sentence, nobody deletes
one, and after a year the docblock is a sedimentary record of every refactor
with the current contract somewhere in the middle.

---

## Closing principles

- **Comments preserve intent.** Code expresses behaviour; comments express why.
- **Coverage is mandatory; length is earned.** Both, always.
- **Describe the present.** Anything else is archaeology, and it rots silently.
- **Say it once, assertively.** If a sentence can go without losing a fact, it
  should already be gone.
