# Catching AI-generated code

AI-assisted coding tools are common and produce code that can look correct while being subtly
wrong in ways that are hard to catch. There's a compounding risk: AI learns from existing code,
including this codebase, so the more AI-generated mediocrity gets merged unreviewed, the more it
becomes the pattern AI amplifies in future suggestions. Catching it early is a force multiplier.

The point is **not** to detect-and-shame. It's to be more alert to the specific failure modes AI
tends to introduce, and to **hold the same standard regardless of whether code was written by a
human or an AI.**

## What to look for

- **Plausible but meaningless names.** Identifiers and comments that sound reasonable but
  communicate nothing specific: `processedData`, `handleResult`, `updatedValue` with no concrete
  context; comments that restate *what* the code does rather than *why*.
- **Boilerplate that doesn't fit.** Large blocks of defensive code, exhaustive null checks,
  verbose try/catch wrappers, and generic error handling that appear regardless of whether the
  context actually requires them.
- **Wrong-level abstractions.** AI pattern-matches on what it has seen, producing code with the
  right *structure for a different problem*: a helper that looks like a generic utility but
  contains application-specific logic; a component following a library pattern in a context where
  that pattern doesn't apply.
- **Inconsistent style or idiom.** Code matching the style of whatever the model was trained on
  rather than this codebase's conventions — inconsistencies in how errors are handled, how state
  is managed, or how modules are structured relative to the rest of the project.
- **Hallucinated APIs.** AI will confidently call functions or access properties that don't
  exist. If a code path calls an unfamiliar method on a library you know well, **verify it
  exists** (check the docs or the installed version) before assuming it's fine.

## The right response

Don't be accusatory. Open a conversation and keep the focus on the code:
- If it's not idiomatic: *"This pattern doesn't match how we do X in this codebase — can you walk
  me through your intent here?"*
- If it's correct but not idiomatic, say so and let the author decide.
- If it's wrong, say so plainly, with the specific mechanism — same as any other finding.

In the report, AI-smell findings should be phrased as Questions or Suggestions unless they
correspond to an actual correctness/contract/security problem (in which case label them by the
real defect — Blocking — not by the suspicion of AI authorship).
