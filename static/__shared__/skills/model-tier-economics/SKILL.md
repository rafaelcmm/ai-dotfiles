---
name: model-tier-economics
description: >-
  Where a cheaper model saves money and where it quietly costs more: which dispatched
  roles can take a cheaper tier, which decisions cannot, and the evidence bar a tier
  change has to clear. Use this skill when choosing a model or reasoning effort for a
  subagent, when configuring a reviewer or implementer agent's tier, when a run's cost
  needs reducing, when someone proposes moving work to a cheaper model, or when deciding
  what a cheap model may be trusted to decide.
---

# Model tier economics

**Cheapen mechanics, never judgment.**

That sentence governs every decision below. A tier change is acceptable when the work it
moves is *mechanical* — deterministic, scriptable, or cheaply verifiable after the fact.
Judgment stays at the top tier or with the user.

## Turn count beats token price

Price per token is not the cost. The cost is price per token × the number of turns, and
each turn re-reads the whole context.

Their measurement: **cheap models took 2–3× the turns on multi-step work and cost more
overall.** In the run that quantified it, **678 of 1,197 subagent turns were on the
cheapest tier and the result was a cost *regression*** — 42.8 min / 14.5M tokens became
69.9 min / 32.2M tokens at equal quality.

### Tier floors

| Role | Floor | Why |
|---|---|---|
| Reviewer of any kind | **Mid-tier**, never the cheapest | Reviewing is judgment end to end: severity calibration, spec verdicts, knowing what not to flag. See the two deaths below. |
| Implementer working from prose | **Mid-tier** | Prose requires interpretation, and interpretation is where cheap models burn turns. |
| Implementer whose task text contains the complete code | **Cheapest tier** | The work is transcription plus running the tests. This is the one clean cheap-tier case. |
| Single-file mechanical edit (rename, constant, field addition) | **Cheapest tier** | No interpretation, verifiable by diff. |
| Multi-file work with integration concerns | **Mid-tier** | Coordination across files is judgment about interfaces. |
| Design decisions, architecture, final whole-branch review | **Top tier** | The expensive error here is invisible until much later. |
| Fix-loop rounds 4–5 | **One tier above the implementer that got stuck** | The capability bump is the point of the escalation. |

Set **model and reasoning effort explicitly on every dispatch**. An omitted model inherits
the session's — usually the most expensive — and setting a model without an effort resets
effort to that model's default. See `subagent-dispatch`.

## The judgment points that stay at the top tier

These are not negotiable per-task. Enumerate them, and if a proposal moves any of them
down, the proposal has to either make the decision once at plan time with the expensive
model, or add an explicit rule that routes the decision back up at execution time:

- **`BLOCKED` / `NEEDS_CONTEXT` diagnosis** — why a worker is stuck, and which remedy.
- **"Cannot verify from the diff" adjudication** — resolved with cross-task context the
  reviewer never had.
- **Dispatch curation** — resolving ambiguity and drawing task boundaries. Measured
  load-bearing in their runs: one curated note on a single task prevented a wrong
  implementation outright.
- **Severity calibration** — what is Important versus Minor.
- **False-positive adjudication** — deciding a finding is wrong.
- **Recognising that the plan itself is wrong**, not the implementation.
- **Recognising when to escalate to the user.**

## The two measured deaths

Both were pre-registered experiments that failed their gates. Their numbers, not ours.

**Cheap reviewers (died).** With the cheapest tier forced on per-task reviewers, planted
defects ×5: 2 pass / 1 indeterminate / 2 fail against a 5/5 baseline. At the finding level,
**the cheap reviewers flagged 0 of 10 planted defects at correct severity** — one found and
downgraded using the exact rationale the prompt prohibited, nine missed or rationalized: a
**DRY violation praised as YAGNI**, an **assert-nothing test called plan-compliant**. The
conclusion they drew is the one worth carrying: **cheap reviewers fail by *advocating* for
defects.** The runs that passed did so on redundancy elsewhere, not on the reviewer.

**A cheap controller (died).** The cheap controller's mechanics were clean and the cost
looked good. Then the quality gate **collapsed into plan-compliance advocacy** — "no
assertion, as required" listed under Strengths — and **the defect shipped in 4 of 5 runs**.
The decisive detail: **the same mid-tier reviewers under an expensive controller caught
that defect 5 of 5 times.** The reviewers were not the variable. The dispatcher's judgment
was.

One nuance they measured and we keep: a cheap controller *does* escalate an **explicit**
contradiction (5/5 on a plan that openly contradicted itself). What it absorbs silently is
**implicit authority-versus-quality adjudication** — the case where the plan mandates
something the quality bar calls a defect.

## The evidence bar for any tier change

- **N=5, never N=1.** "Single-run gates were this campaign's weakest methodology."
- **"The cheap model usually gets it right" is not acceptance evidence.** Judgment failures
  are rare-event, high-blast-radius, and largely invisible to pass/fail gates — which is
  why a tier change needs a judgment audit (interrogate every escalation and adjudication
  event in the gate runs against the expensive baseline) on top of the N=5 scenario gates.
- **A silently absorbed judgment call fails the change** regardless of the scenario
  verdicts. The cheap model resolving what it should have escalated is the failure, even
  when its resolution happened to be right.

`evaluation` covers rubric and test-set mechanics; this section is the acceptance bar
specific to moving work down a tier.

## A pattern that works: two-phase cheap-categorize, expensive-analyze

Split a judgment task into a mechanical pass and a judgment pass. The cheap tier
categorizes, filters, extracts, or counts — deterministic work with a checkable output —
and the expensive tier analyzes only what survived. The cost falls with the volume the
expensive model reads, and no judgment moved down a tier.

This is the legitimate shape of a cheap-tier win. The illegitimate shape is handing the
same decision to a cheaper model and hoping.

## Dead ends

Recorded so they are not re-proposed. Each was tested and killed:

- **Dispatch-time batching of judgment tasks** — one worker handling several plan tasks.
  Kills the fresh-context property and coarsens every gate to the batch boundary.
  (Batching *small same-shape mechanical edits* is a different thing and is fine — see
  `subagent-dispatch`.)
- **Turn-discipline mechanisms** — instructing the dispatcher to batch its own tool calls
  or pipeline parallel calls. It emitted exactly one tool call per message in every run;
  46% of its turns were thinking and narration, a **prompt-immune floor**.
- **Scoped re-reviews as a cost lever** — worth ~$0.50 a run at most. They are worth doing
  for convergence (see `review-fix-loop`), not for dollars.
- **Their own "better guidance cut the cost" claim, retracted.** An elicited plan ran at
  $6.34–8.49 while the no-guidance control hit $7.59–7.73 — inside the same range. **The
  win belonged to better plans, not to the guidance.** They re-attributed the claim to
  fidelity and variance instead. Cite it that way.

## Common Rationalizations

| Excuse | Reality |
|---|---|
| "Cheaper tier, cheaper run" | Cheap models took 2–3× the turns; 678 of 1,197 cheap turns was a cost regression at equal quality. |
| "A review is just reading a diff — mechanically cheap" | Mechanical cheapness is not mechanical judgment. Cheap reviewers flagged 0 of 10 planted defects at correct severity. |
| "The cheap model gets this right almost every time" | Judgment failures are rare-event and high-blast-radius, and pass/fail gates do not see them. That sentence is not acceptance evidence. |
| "It passed the gate run" | One run is this campaign's weakest methodology. N=5, plus a judgment audit. |
| "The dispatcher just routes — it can be cheap" | The cheap controller shipped the defect 4/5 while the same reviewers under an expensive one caught it 5/5. |
| "I'll leave the model off this dispatch, the default is sensible" | The default is inherited, not chosen. Name model and effort every time. |
| "One worker can take three tasks and save two dispatches" | That is dispatch-time batching of judgment work — tested, killed, and it coarsens every gate. |
| "Our guidance made runs cheaper last time" | They measured that and retracted it: the win was the better plans. |

## Related

- `subagent-dispatch` — the dispatch rules these tiers attach to.
- `review-fix-loop` — the escalation that depends on a tier bump.
- `evaluation` — rubric and test-set mechanics for running the gates this skill requires.
- `context-optimization` — token-level cost theory; this skill is about which model does which job.

## Attribution

Adapted from `obra/superpowers` (MIT, © 2025 Jesse Vincent). Every number here —
2–3× turn inflation, 678/1,197 turns, 0 of 10 planted defects, 4/5 versus 5/5, the
retracted cost claim — is **their measurement**, reported from their experiment ladder and
not reproduced by us.
