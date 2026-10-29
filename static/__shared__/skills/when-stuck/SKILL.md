---
name: when-stuck
description: >-
  Use when stuck on a design or implementation problem and unsure which
  angle to try next — the same thing keeps getting implemented multiple
  ways, special cases keep accumulating, conventional approaches feel
  inadequate, the same shape keeps showing up in unrelated places, a
  solution feels forced by an unquestioned assumption, or scale/edge-case
  behavior is unclear. Not for a broken test or wrong output — that is a
  debugging problem, not a stuck-on-approach problem.
---

# When Stuck

Different stuck-types need different techniques. Match the symptom below
to a technique, apply it, and if still stuck after one full pass, try a
different row rather than repeating the same one harder.

## Dispatch table

| Symptom | Technique |
|---|---|
| Same thing implemented 5+ ways, growing special-case list | Simplification cascades |
| Same shape keeps showing up in 3+ unrelated places, "haven't we done this before" | Meta-pattern recognition (below) |
| Stuck on "this must be done this way", can't question the premise | Inversion exercise |
| Will this hold at 1000x scale, or at 1/1000th? Edge cases unclear | Scale game |
| Conventional approaches feel inadequate, want a different angle | Collision-zone thinking (below) |
| Oscillating between two approaches that are each right in different contexts | Preserving productive tensions (below) |
| Code is broken — wrong output, failing test | Not this skill. Use your debugging process. |
| "Why does this exist" on code you're about to delete or simplify | Not this skill. Use `chestertons-fence` — it is stronger and mandatory already; this skill does not duplicate it. |

## Simplification cascades

**Core principle:** one insight that eliminates multiple components —
"if this is true, we don't need X, Y, or Z." Ten-times wins, not
ten-percent improvements.

| Symptom | Likely cascade |
|---|---|
| Same thing implemented 5+ ways | Abstract the common pattern |
| Growing special-case list | Find the general case that absorbs them |
| Complex rules with exceptions | Find the rule that has no exceptions |
| Excessive config options | Find the default that works for 95% |

Process: list every variation that currently exists → ask "what if they're
all the same thing underneath?" → extract the domain-independent
abstraction → test it against every variation, not just the easy ones →
measure the win by what you can now delete, not by what you added.

Red flag you're missing one: "we just need to add one more case" repeating
indefinitely, or a part of the code nobody wants to touch because it's
"complicated" — complexity often hides an uncollapsed pattern. This
complements `objective-files`: a cascade is frequently the reason a file
grew past its single responsibility in the first place.

## Scale game

**Core principle:** extremes expose fundamental truths hidden at normal
scale. Test both directions.

| Dimension | Test at extremes | Reveals |
|---|---|---|
| Volume | 1 item vs. 1B items | Algorithmic complexity limits |
| Speed | Instant vs. 1 year | Async/caching requirements |
| Users | 1 vs. 1B concurrent | Concurrency and resource limits |
| Duration | Milliseconds vs. years | Memory leaks, unbounded state growth |
| Failure rate | Never fails vs. always fails | Whether error handling is adequate or decorative |

Process: pick a dimension → push it 1000x smaller, then 1000x bigger →
note what breaks at each end → note what survives unchanged at both —
that's the part that was never actually scale-dependent.

Red flag you need this: "it works in dev," "should scale fine" with no
test behind the claim, or being surprised by production behavior you
could have predicted by pushing either extreme.

## Inversion exercise

**Core principle:** flip the assumption and see what still works. The
opposite sometimes reveals the truth the original framing hid.

| Normal assumption | Inverted | Reveals |
|---|---|---|
| Cache to reduce latency | Add latency deliberately | Debouncing patterns |
| Pull data when needed | Push data before needed | Prefetching, eager loading |
| Handle errors when they occur | Make errors impossible | Type systems, contracts |
| Optimize for the common case | Optimize for the worst case | Resilience patterns |

Process: list the assumptions you're treating as fixed → invert each one
→ work out what you'd do differently under the inversion → keep only the
inversions that actually hold up somewhere, not all of them.

Red flag you need this: "there's only one way to do this," or a solution
that feels forced and you can't articulate why it's necessary beyond
"that's just how it's done."

## Meta-pattern recognition

When the same shape appears in 3+ domains, it's likely a universal
principle, not a coincidence: spot the repetition, strip it to its
domain-independent form, name the variation points, then check where else
the abstract form applies. ("Rate limiting" in API throttling, traffic
shaping, and circuit breakers is the same shape as an LLM token budget —
bound resource consumption to prevent exhaustion.) Red flag: "this problem
is unique" is usually wrong, and multiple people independently solving
"different" problems identically is the tell.

## Collision-zone thinking

When conventional approaches feel inadequate, deliberately force two
unrelated concepts together: "what if we treated X like Y?" Pick two
domains, force the combination, work out the emergent properties, then
test where the metaphor breaks — it always breaks somewhere, and where it
breaks is informative too. (Treating distributed services like electrical
circuits surfaces circuit breakers and fuses; it also reveals circuits
don't have retry logic, which is itself useful to know.) This is the
thinnest technique here — reach for it only after the rows above have not
helped, since forced metaphors can also waste a pass on nothing.

## Preserving productive tensions

Not every disagreement needs resolving. A tension is productive — worth
preserving, not solving — when both approaches optimize for different
*valid* priorities (cost vs. latency, simplicity vs. features), the better
choice depends on deployment context rather than technical superiority,
and the trade-off is real and won't dissolve under cleverer engineering.
Force resolution instead when preserving both is prohibitively expensive
to build or maintain, the approaches fundamentally conflict, one is
objectively superior for this specific context, or the choice is a
one-way door. Preservation patterns: make the choice configurable, keep
both as parallel implementations behind one shared contract, or document
the trade-off explicitly with its resolution trigger. Red flag you're
forcing resolution prematurely: "we need to pick one" with no stated
reason, or resolving a tension "to make progress" when preserving it *is*
the progress.

## Combining techniques

- Simplification + meta-pattern: find the pattern first, then simplify
  every instance of it.
- Collision + inversion: force the metaphor, then invert its assumptions.
- Scale + simplification: extremes reveal what was never load-bearing,
  which is exactly what the cascade should eliminate.

## Attribution

These techniques derive from agent patterns in Microsoft Amplifier
(`github.com/microsoft/amplifier`), adapted as skills by `obra/superpowers`
(MIT, © 2025 Jesse Vincent) and consolidated here from eight near-identical
skills into one dispatch table, per our own file-economy rules.
