# Micro-testing skill wording

Part of `writing-skills`. The empirical protocol for finding out whether a piece of guidance
actually changes what an agent produces — cheap enough to run per phrasing, before anything
ships.

## Contents

1. [What this is for](#1-what-this-is-for)
2. [Start with the control](#2-start-with-the-control)
3. [Harness shape](#3-harness-shape)
4. [Sampling](#4-sampling)
5. [Scoring: read every match](#5-scoring-read-every-match)
6. [Variance is a metric](#6-variance-is-a-metric)
7. [Worked example](#7-worked-example)
8. [Pressure scenarios: discipline skills only](#8-pressure-scenarios-discipline-skills-only)
9. [Common mistakes](#9-common-mistakes)
10. [Attribution](#10-attribution)

---

## 1. What this is for

Micro-testing answers one question: does this wording move the output, compared to no wording
at all? It is for guidance whose purpose is to change behaviour — a recipe, a prohibition, a
required slot, a conditional.

It is not for pure reference skills. An API reference has no behaviour to shift; test those by
checking that an agent can find the right section and apply it.

A single sample tells you nothing, and neither does your own reading of the text. Authors
cannot evaluate their own wording: they already know what it means.

## 2. Start with the control

**Always run a no-guidance control arm.** Same task, same surrounding context, the guidance
removed entirely.

**If the control does not exhibit the failure, stop. There is nothing to fix.** Do not write
the guidance. Prose you add for a failure the model no longer makes still costs tokens on
every load, still constrains future behaviour, and still has to be maintained — and you will
never be able to measure whether it helps.

This is not a formality. A ten-task placeholder experiment ran 40 samples across four variants
including the control: 40/40 clean, the sole regex hit being a self-review attesting that no
placeholders were present. The correct outcome was to leave the skill untouched and open no
change at all.

The control also distinguishes the two prohibition outcomes. In one measured pair it exposed a
backfire (prohibition scored worse than silence on output shaping) and, in the same series, a
prohibition that worked (3/5 control failures against 0/5 with the directive). Without the
control arm, both look identical: "the guidance produced some failures".

## 3. Harness shape

One API call per sample, nothing stateful between samples:

- **System prompt** = the guidance variant *inside the realistic context it will live in* —
  the full skill, or the full prompt template. Never the guidance in isolation; isolation
  removes the competing incentives that cause the failure in the first place.
- **User message** = a realistic mid-workflow task that tempts the failure. Under-specify the
  input where under-specification is what triggers the behaviour you are hunting.
- **Output** = the composed artifact: the dispatch prompt, the plan, the report, the review.

Run it as a direct API call where you can. A single-shot subagent per sample works too,
provided each one starts from fresh context — reusing a context contaminates every sample
after the first.

Keep a fixture task file per experiment and reuse it across variants. Changing the task
between arms makes the comparison meaningless.

## 4. Sampling

- **5 or more reps per variant.** Below five you are reading noise.
- **One fresh context per rep.** No carry-over, no follow-up turns.
- **Default temperature.** You are measuring the behaviour that will actually ship.
- **The model that does the job in practice.** Guidance tuned on one tier can fail on another;
  if a cheaper tier will run this skill, sample that tier too.
- **Every arm on the same day, same model build.** Comparing across model versions measures
  the model, not the wording.

Cost is roughly $0.15–0.30 per sample and seconds per iteration, against hours and tens of
dollars for a full end-to-end eval run. Iterate phrasings here; promote to a full run only
when the change is structural.

## 5. Scoring: read every match

Score programmatically for speed — grep for unambiguous markers, count the artifacts that
lack a required element — and then **read every flagged match by eye before trusting any
verdict.**

Two failure modes make automated counts lie in both directions:

- **Template echoes.** The artifact repeats a header or a label from the template, and the
  regex counts it as content.
- **Quoted counter-examples.** The agent quotes the prohibition back, correctly, while
  complying. Naive negation detection scores that as a violation.

In one measured run, both occurred in the same batch: one "violation" was the controller
quoting the rule it was obeying.

Pick metrics that are countable and directional before you look at any output. Decide which
way is better in advance:

| Metric | Direction |
|---|---|
| Banned-token count | Lower better |
| Required elements present per artifact | Higher better |
| Steps lacking a runnable command with expected output | Lower better |
| References to symbols defined nowhere in the artifact | Lower better |
| Artifact length against the stated target | Closer better |

Deciding after the fact lets you fit the metric to the result you hoped for.

## 6. Variance is a metric

Report spread, not just the mean. When guidance lands, reps converge: the winning recipe in
the dispatch experiment scored 3.0 with **zero variance** across five reps.

Five reps producing five different interpretations means the wording is not binding, whatever
the average says. That is a signal to tighten the form — usually to restate it as a positive
recipe or a structural slot — **not** to add more words. Adding a qualifier to a converged
recipe is the one intervention measured to make things worse: 3.0 with zero variance became
3.8 and noisy.

Acceptance rule: adopt a variant only when it beats the incumbent on the primary metric
*without* losing ground on the others. A variant that halves banned tokens while dropping
code-block coverage has not won.

## 7. Worked example

Guidance under test: how to stop a controller from re-typing spec values into a dispatch
prompt it composes for a subagent.

1. **Fixture.** A brief with a handful of concrete, quotable values, plus a dispatch task that
   makes re-typing them feel helpful.
2. **Arms.** Control (no guidance); prohibition ("don't restate the brief"); positive recipe
   ("your dispatch contains: (1)…(5)"); recipe plus a nuance clause.
3. **Metric.** Count of spec values needlessly re-typed. Lower better.
4. **Reps.** 5 per arm, fresh context, default temperature.
5. **Result.** Recipe 3.0 (zero variance) · control 3.6 · prohibition 4.4 · recipe+nuance 3.8
   (noisy). Distributions fully separated.
6. **Decision.** Adopt the recipe. Record that the prohibition was worse than silence, so
   nobody reintroduces it as the "stronger" option.

Write the result down with its numbers. An undocumented experiment gets re-run, or worse,
re-argued from intuition.

## 8. Pressure scenarios: discipline skills only

Micro-tests verify wording. They do not verify that an agent keeps a rule when keeping it is
expensive. For a discipline skill, that needs a pressure scenario — and those are slow and
costly, so spend them only where the failure is a discipline failure.

A pressure scenario is realistic, forces a choice, and stacks three or more pressures: time,
sunk cost, authority, exhaustion, social cost, "being pragmatic". It names real paths and real
constraints, offers concrete options rather than an open question, and leaves no escape into
"I'd ask first".

A single pressure is usually resisted. Combinations are what break through.

Capture every rationalization verbatim. Those exact sentences are the rows of the
rationalization table described in `bulletproofing.md`, the sibling reference — a paraphrase closes nothing.

When an agent violates the rule while holding the skill, ask it directly how the skill should
have been written to make the right choice unmistakable. Three answers, three different fixes:
"the skill was clear, I ignored it" needs a stronger foundational principle; "it should have
said X" takes X verbatim; "I didn't see that section" is a layout problem, so move the point
earlier.

## 9. Common mistakes

| Mistake | Why it ruins the result |
|---|---|
| No control arm | You cannot tell a working rule from a rule fixing nothing. |
| Guidance tested in isolation | Removes the competing incentive, so the failure never appears. |
| One or two reps | Within-arm spread is larger than most between-arm differences. |
| Trusting the grep count | Template echoes and quoted counter-examples score as hits. |
| Changing the task between arms | Measures the task, not the wording. |
| Choosing metrics after seeing output | Fits the measurement to the hoped-for answer. |
| Fixing a noisy variant by adding words | Measured to degrade a converged recipe. Re-derive the form instead. |
| Pressure-testing a shaping failure | Expensive and off-target; shaping shows up in ordinary samples. |

## 10. Attribution

Adapted from the micro-testing method and measurements in
[obra/superpowers](https://github.com/obra/superpowers) (MIT, © 2025 Jesse Vincent).
