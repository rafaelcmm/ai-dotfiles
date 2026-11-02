# Surviving a flaky upstream on the request path

For the cases where fetching card data at request time is genuinely right — prices, stock,
user-specific data — or where you have not yet moved to a snapshot.

Read the first section before building any of this. Most of the machinery here is only worth
having if moving the data off the request path is genuinely impossible.

## Contents

- [First, check you need any of this](#first-check-you-need-any-of-this)
- [Never cache a failure](#never-cache-a-failure)
- [Bound the retry rate, separately](#bound-the-retry-rate-separately)
- [A circuit breaker for the host, not the record](#a-circuit-breaker-for-the-host-not-the-record)
- [Single-flight the recovery probe](#single-flight-the-recovery-probe)
- [Report the outage once](#report-the-outage-once)
- [Degrade to something, never to an error](#degrade-to-something-never-to-an-error)
- [Fetch the corpus, not the record](#fetch-the-corpus-not-the-record)
- [Cap the page loop independently](#cap-the-page-loop-independently)
- [State the cost of the fallback](#state-the-cost-of-the-fallback)
- [What this looks like when it is done](#what-this-looks-like-when-it-is-done)

## First, check you need any of this

Everything below is machinery whose only purpose is to make a third party survivable. For
immutable card identity, the cheaper answer is to move the data into a committed snapshot and
delete all of it.

The question to answer honestly: **does this field change after the card is printed?** Name,
type, cost, rarity, rules text, artwork, set, collector number — no. Price, stock, a user's
collection — yes.

If the answer is no for everything on the page, stop here and build the snapshot instead.

## Never cache a failure

Cache successes only.

The mechanism, with a function-wrapping cache: make the loader **throw** on a miss or an error,
and catch the throw outside the cache. A function that throws is not cached, so nothing is
stored, and the next request simply retries.

```js
const load = cache(async () => {
  const record = await fetchRecord(id);
  if (!record) throw new Unavailable(id);   // throw, so nothing is stored
  return record;
});

try {
  return await load();
} catch {
  return null;                               // caller degrades
}
```

Why this matters more than it looks: a card added upstream tomorrow resolves on the next
request, with no manual invalidation, no tag-based purge and no revalidation endpoint. The
staleness problem self-heals. Caching a negative result means a newly added card stays
invisible for the whole entry lifetime, and the only fix is an operator action nobody
remembers to take.

Make the no-negative-caching behaviour explicit in the module's documentation. It looks like an
oversight and it is the load-bearing decision.

## Bound the retry rate, separately

Not caching failures has a direct consequence that is easy to miss: **during an outage, every
uncached record is re-fetched on every render.**

Measured 2026-08-31, during an outage where `api.tcgdex.net` refused connections on both its A
and AAAA records, against a page resolving about 160 cards: roughly 160 connect timeouts and
160 thrown errors per page view, against a host already known to be down. The cache was behaving exactly
as designed. What it never bounded was the retry *rate*.

These are two separate problems and they need two separate mechanisms:

| Problem | Mechanism |
|---|---|
| a card added upstream should resolve without manual action | do not cache failures |
| a host that is down should not be asked 160 times per render | bound the retry rate |

Layer one: a **per-record memo**. A record that just failed is not asked again for a few
minutes. This is safe by construction — a failed lookup cached nothing, so skipping it cannot
bypass good cached data.

Pick the window against the failure it exists for. Five minutes is well matched to a
multi-hour host outage while staying far below anything that feels like caching: a card
genuinely added mid-session still appears within five minutes, against a success cache
measured in days.

Key the memo by every dimension that can fail independently — language as well as id, since a
card present in English and missing in Portuguese must not suppress the English attempt. Sweep
expired entries past a size threshold so a crawler walking malformed ids cannot grow the map
without limit.

## A circuit breaker for the host, not the record

Layer two. The distinction is the entire safety property:

- **"This id does not exist"** says nothing about any other id in the catalogue. It must never
  open the breaker, or three mistyped ids take the whole catalogue offline.
- **"The connection was refused"** says everything. Open the breaker for every id.

So only a **transport** failure — connection refused, DNS failure, timeout — opens it. A clean
404 never does.

```js
export function reportTransportFailure(now) { breakerOpenUntil = now + WINDOW_MS; }
export function reportUpstreamSuccess()     { breakerOpenUntil = 0; probeOutstanding = false; }
```

Per-process state is fine. It resets on deploy and each instance keeps its own; the worst case
is one extra probe per instance.

## Single-flight the recovery probe

Without this the breaker is useless in practice.

A page resolving its cards in one `Promise.all` has ~160 callers arriving simultaneously. When
the breaker's window closes they all see a closed window, all probe at once, and reproduce the
original storm exactly once per window.

Elect exactly one prober. Everyone else keeps skipping until it reports back:

```js
export function acquireUpstreamSlot(now) {
  if (breakerOpenUntil === 0) return "call";
  if (now < breakerOpenUntil) return "skip";
  if (probeOutstanding) return "skip";
  probeOutstanding = true;
  return "probe";                 // this caller MUST report the outcome back
}
```

Three states, not two: `call` when healthy, `probe` for the one elected caller, `skip` for
everyone else. The elected caller must always release the probe, including on an exception, or
the breaker jams open.

## Report the outage once

The useful signal during an outage is a single "the upstream is down and the fallback is
covering" — not one line per record.

Logging per record turned a fixed 160-error storm into a 160-warning one: same volume,
different severity, and it buries whatever else the developer was reading. Rate-limit the
report to once per breaker window.

## Degrade to something, never to an error

Every consumer must handle the null case by rendering something usable.

- A card page with no upstream data renders the standard layout, not a 500.
- A listing with no metadata renders with whatever local data exists and no facets, rather
  than disappearing.
- An image with no resolvable URL renders a placeholder sized to the frame, not a broken
  image icon.

**Never throw from a resolver that a page renders.** Return null, log once, and let the caller
decide. The exception is a malformed id, which is a configuration error worth surfacing loudly
because no retry will fix it — validate ids separately from fetching them, and let validation
throw while fetching returns null.

## Fetch the corpus, not the record

Counter-intuitive and usually right: when a page needs many cards, loading the whole upstream
catalogue once beats N per-card lookups.

A listing page showing 250 cards — a larger surface than the 160-card page above — is 250
separate cached fetches on a cold cache. Sweeping the
catalogue is 15 paged requests, gives every card at once, and yields the facet vocabularies for
free — which no single-card endpoint can provide at all.

Keep a per-card fallback for the miss path, so a card added upstream after the corpus was
cached is still resolvable within its own window.

Memoise the composed result at module level alongside whatever cache the individual fetches
use. The fetches are cached; the composition into maps and vocabularies is not, and redoing it
per request in a warm process is pure waste.

## Cap the page loop independently

```js
const lastPage = Math.min(reported.pages ?? 1, MAX_PAGES);
```

An upstream-reported page count must never bound a loop unchecked. A malformed response
reporting an implausible count turns one sweep into thousands of outbound requests against a
free service. Set the cap at a few times the current real page count, and **warn when it
truncates** so the condition is visible rather than silent.

## State the cost of the fallback

Write down what degrading actually loses, in the module that degrades.

A worked example: while a breaker is open, a card whose data *is* cached gets answered from
the local fallback instead — because the breaker skips the loader, and there is no way to read
the cache without running it. That trades some fidelity (upstream-only fields: variants,
legality, illustrator) for a page that renders fast and quietly during an outage, and it
self-corrects within one window of the host recovering.

That is a real cost and a defensible trade. What is not defensible is discovering it during an
incident because nobody wrote it down. State the cost next to the mechanism.

## What this looks like when it is done

```
validate id            -> throws on malformed (configuration error, surface it)
check per-record memo  -> skip if it failed recently
acquire breaker slot   -> call | probe | skip
  skip                 -> return fallback immediately
  call / probe         -> cached loader, which throws on miss so nothing is stored
    success            -> close breaker, return record
    transport failure  -> open breaker, report once per window, return fallback
    clean miss         -> remember the per-record miss only, return fallback
release probe          -> always, including on exception
```

Roughly 200 lines of mechanism. All of it exists because a third party has no uptime
commitment — which is the argument for the committed snapshot, and the reason this reference
opens by telling you to check whether you need any of it.
