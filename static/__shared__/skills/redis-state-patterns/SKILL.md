---
name: redis-state-patterns
description: >-
  Decide whether server state actually needs Redis, and implement either choice
  well — in-process caches and rate limiters with bounded growth and an honest
  multi-instance caveat, or Redis with sound key design, argued TTLs, and locks.
  Use this skill when the task involves "add a cache", "rate limit this
  endpoint", "do I need Redis/Upstash/KV", "cache invalidation", "distributed
  lock", "memory keeps growing", "cache key design", "TTL", "state across
  serverless instances", or any in-memory `Map` that outlives a request.
---

# Redis state patterns

Most "we need Redis" turns out to be "we need a cache", and most caches do not
need to be shared. The decision comes first; the implementation details below
matter only after it.

## The decision rule

**An in-process `Map` is correct when per-instance duplication costs extra
upstream calls and never wrong data.** Each instance independently warms its own
copy. A cold start or a fan-out costs a few more upstream requests. Nothing a
user can observe is inconsistent.

**Redis is required when correctness depends on instances agreeing:**

- Locks and mutexes.
- A rotating credential — two instances refreshing concurrently destroy it.
- Idempotency keys for at-least-once delivery.
- A *global* rate limit, as opposed to a best-effort one.
- Anything a user can observe as inconsistent between two requests.

**Neither, when the framework already has it.** Derived, regenerable, PII-free
data with a staleness budget is what a framework data cache (Next's
`unstable_cache`/`'use cache'`, request-scoped `cache()`) exists for. Reaching
for an external store there adds a dependency, a failure mode, and a bill for
nothing. If a project already has Redis credentials for something that genuinely
needs mutable shared state, say explicitly in the new module that it must not
import them — otherwise their presence becomes an argument for using them.

## The honesty discipline

This is the part that actually gets skipped, and the part worth enforcing.

When you choose in-process, write the caveat **into the module's own
documentation**: state that each instance keeps its own copy, name what that
costs, and name this module as the seam for the upgrade. Then **do not build the
upgrade**.

```ts
/**
 * Scaling caveat, stated deliberately and NOT solved here: state lives in a
 * plain in-memory Map in this module, so a multi-instance deploy gets one
 * independent cache per process. That costs extra upstream calls after a cold
 * start or a fan-out — it never serves wrong data. If volume ever justifies it,
 * swap the Map for a shared store behind these three functions and nothing
 * above them changes.
 */
```

A documented caveat is a design decision. An undocumented one is a latent bug,
and the next reader cannot tell which they are looking at. The three-sentence
form — what it is, what it costs, where the seam is — is the whole requirement.

## Bounded growth is mandatory

Any in-process store on a public endpoint must bound its size. This is the most
common real defect in otherwise-fine cache code.

The failure: a `Map` keyed on something visitor- or attacker-supplied — a client
IP, a postal district, a cart shape — grows without limit, because **a key seen
once is never read again and therefore never evicted by a read-path expiry
check**. Eviction-on-read only collects keys somebody asks for twice. The long
tail is exactly what you never collect.

Two mechanisms, both adequate; pick one and say why.

**Opportunistic sweep** — for a limiter whose entries all expire on the same
schedule:

```ts
callsSinceSweep += 1;
if (callsSinceSweep >= SWEEP_INTERVAL) {        // e.g. 1000
  callsSinceSweep = 0;
  for (const [k, s] of store) {
    if (now - s.windowStart >= windowMs) store.delete(k);
  }
}
```

O(size) when it runs, but amortizes to a small constant per call and needs no
extra bookkeeping — no LRU ordering, no secondary size index.

**Hard cap with FIFO eviction** — for a value cache where entries are larger:

```ts
if (!store.has(key) && store.size >= MAX_ENTRIES) {
  const oldest = store.keys().next().value;     // Map preserves insertion order
  if (oldest !== undefined) store.delete(oldest);
}
store.set(key, { value, expiresAt: Date.now() + TTL_MS });
```

True LRU is usually **not** worth its bookkeeping here: staleness is already
bounded by the TTL regardless of eviction order, so FIFO-by-insertion costs
nothing but a slightly lower hit rate on hot keys. Say that in the comment, so
the simplification reads as a decision rather than an oversight.

## Key design: truncate to the granularity that matters

Include the data's real granularity in the key, and **no finer**.

Worked example — caching a shipping quote by destination postal code. Key on the
**first five digits only**, never the full code. Five digits identify a whole
delivery district, which is the granularity the carrier prices on anyway. Two
things improve at once:

- **Hit rate**: the cache hits across different shoppers in the same district.
- **Retention**: a cache key is the longest-lived artefact of a request. A full
  postal code sitting in one would be personal data resident in memory long
  after the request that supplied it.

Generalize: truncate a key to the precision at which the answer actually varies.
It raises the hit rate and reduces what you store about a person in the same
move — the rare case where privacy and performance point the same way.

Two more rules from the same example:

- **Sort any multi-part signature.** A cart whose lines arrive in a different
  order is the same parcel; an unsorted signature misses the cache for no reason.
- **Include every input that changes the cached result, even ones that do not
  change the request.** A flag that shifts a computed delivery date must be in
  the key, or two otherwise-identical requests collide and the second caller is
  served the first's date. Omitting one of these is a defect, not a
  simplification — and the symptom is a plausible wrong answer, which is the
  worst kind.
- **Exclude what is commercially sensitive and does not affect the result.**

## TTL must be argued, not rounded

Derive the TTL from how fast the underlying truth moves.

> Carrier price tables change on the order of months, while the same handful of
> destinations are quoted repeatedly across shoppers. Six hours is well inside
> the interval over which the answer can actually change, and short enough that
> a mid-day price change is live by the same evening.

That is the shape of the argument: name the rate of change upstream, then show
the TTL sits inside it. "5 minutes" with no reasoning is a number nobody can
later revise with confidence.

**Never cache a failure.** Storing a `null` from a failed upstream call pins the
outage in place for the full TTL after the upstream recovers.

## Lock pattern

```ts
const lock = await redis.set(LOCK_KEY, 1, { nx: true, ex: TTL_SECONDS });
if (lock !== "OK") return degradedPath();   // loser degrades; never queues
```

- The loser **degrades**, it does not wait. Waiting converts one slow critical
  section into a request pile-up.
- TTL covers the critical section plus its write, and no more — long enough not
  to expire mid-operation, short enough that a crashed holder does not wedge the
  resource until someone notices.
- Consider letting it expire rather than deleting it, when a late arrival should
  read the freshly written value instead of retrying the operation.

## Fixed vs sliding window rate limits

A fixed window is simple and cheap, and **permits a burst across the boundary**:
with a limit of 10 per minute, a client can send 10 at 59s and 10 at 61s. Say
this out loud rather than implying a guarantee the code does not provide.

It is an acceptable trade-off for abuse-hardening on a public write endpoint,
where the goal is bounding sustained volume rather than enforcing an exact rate.
It is not acceptable when the limit protects a metered upstream you are billed
for per call, or when a downstream provider enforces a hard rate and will ban
you.

Also state the scope honestly: an in-process limiter is **per instance**. With N
instances the effective limit is N× what the code says. That is fine for
abuse-hardening and wrong for a quota.

## What Redis is not for

- It is not a database. State you cannot regenerate does not belong only there.
- A cache that becomes the source of truth is an outage waiting to happen — if
  losing the store loses data, it was never a cache.
- It is not a queue, a search index, or a lock you can hold across a deploy.

## Checklist

- [ ] Decision recorded: in-process, Redis, or framework cache — and why.
- [ ] If in-process: multi-instance caveat, its cost, and the seam documented in the module.
- [ ] Size bounded by a sweep or a hard cap; mechanism justified in a comment.
- [ ] Key truncated to the granularity that matters; multi-part signatures sorted.
- [ ] Every input affecting the cached value is in the key.
- [ ] No personal data resident in a key longer than necessary.
- [ ] TTL argued from the upstream rate of change.
- [ ] Failures never cached.
- [ ] Locks: `SET NX` + TTL, loser degrades.
- [ ] Rate limiter: window semantics and per-instance scope stated.

## Related skills

- `oauth-token-lifecycle` — the canonical case where shared state is genuinely required.
- `webhook-ingestion-hardening` — idempotency keys, the other common reason for a shared store.
