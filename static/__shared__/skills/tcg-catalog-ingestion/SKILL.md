---
name: tcg-catalog-ingestion
description: >-
  Turn a trading-card-game data source into something an application can serve — choosing
  between request-path fetching, a committed snapshot and a local store, projecting fields,
  versioning the snapshot, writing an idempotent sync script, narrowing untrusted responses,
  and degrading when the upstream is down. Use when ingesting or caching card data, when a
  card API is slow or flaky on the request path, when a sync script must be re-runnable, when
  deciding what to persist versus fetch, or when card art must be stored and served. Load it
  alongside tcg-card-data-sources, which covers what each source returns; this one covers what
  to do with it.
---

# TCG catalogue ingestion

Card data has an unusual property that should drive the whole design: **a printed card is
immutable**. Its name, type, cost, rarity, rules text and artwork will never change. New cards
appear; existing ones do not mutate.

Almost every mistake in this area comes from treating immutable data as if it were live.

## The first decision, and the only one that matters much

Where does card data live at the moment a user asks for it?

| Option | Use when | Cost |
|---|---|---|
| **Fetch on the request path** | the dataset is genuinely dynamic — prices, stock, a user's collection | every request inherits the upstream's latency and uptime |
| **Committed snapshot** — a projected file in the repo, read by static import | card identity, rules text, set metadata, checklists: anything immutable | a new set needs a sync run and a deploy |
| **Local store** — a database or object store filled by a job | the corpus is too large to commit, or several services need it | infrastructure, migrations, a staleness story |

**Default to the committed snapshot for card identity.** It is the option whose failure mode is
"a card released yesterday is missing" rather than "every card page is broken because a hobby
API is down".

The trade is real and should be stated out loud when you take it: with a snapshot, new cards
appear when someone runs the sync, not automatically. That is a known, visible, cheap-to-fix
condition. An upstream outage on the request path is none of those things.

### The progression that keeps happening

Projects arrive at the same place by the same route, and knowing the route lets you skip it:

1. Fetch per card on the request path, with a cache. Works in development.
2. Discover that a cold cache means N upstream requests for one page, and that the upstream
   has no uptime commitment.
3. Add retry, circuit breaking, negative-cache suppression, request deduplication — a pile of
   machinery whose entire purpose is to make a third party survivable.
4. Realise the data never changes, move it to a committed snapshot, and delete all of step 3.

Step 4 is where you want to start. Steps 1–3 are only correct for data that genuinely changes.

## Split immutable identity from volatile fields

The one case where mixing is tempting and wrong: **prices**.

Some card sources now embed pricing (TCGdex does). A price changes daily and carries its own
`updated` timestamp; card identity never changes. Freezing them into one snapshot means either
re-syncing the whole catalogue daily to refresh prices, or serving a stale price as current.

Keep two artefacts on two cadences: identity in the long-lived snapshot, prices in a short-TTL
store or a request-path fetch. Join them on the card id.

## Project, never passthrough

Store the fields you render, not the response you received.

- A source's record is far larger than what any view uses. Lorcana's `allCards.json` is 9.5 MB;
  the ~30 rendered fields are roughly a third of that, and most of the saving comes from
  **omitting absent keys entirely rather than writing nulls**.
- A projection is a declared contract. When the upstream adds a field, nothing changes until
  someone decides to add it — rather than the payload silently growing.
- Name the dropped fields and why, once, in the projection constant. A future maintainer will
  otherwise re-add `externalLinks` without knowing it was excluded deliberately.

Keep the identity and provenance keys even when nothing renders them: the upstream id, the
printed id, and the sync timestamp. They are what make a stale or mismatched record
diagnosable without re-querying the source.

## Version the snapshot

A persisted projection is a **wire format**, not an internal type. Changing it is a data
migration, not a refactor.

- Carry an explicit `v` on the snapshot. One constant, one place.
- A reader that sees an unknown or absent `v` treats the record as unresolvable rather than
  guessing. Guessing produces a half-populated card that looks fine.
- Adding a field is: bump `v`, re-run the sync, commit the data, then ship the reader that
  accepts both versions. In that order — a reader deployed before the data exists reads
  nothing.
- When a sync script cannot import the reader's version constant (a plain `.mjs` script and a
  typed reader, for instance), duplicate the literal with a comment naming the file that owns
  it. Duplication you can grep beats a build step.

## Narrow what you read, including your own data

Every response is untrusted input, and so is a snapshot that a human can edit.

The split that works, applied field by field:

- **Reject the record** when a field it cannot be rendered without is missing or the wrong
  type: the ids, the name, the classification, the art URL, the orientation.
- **Default the rest**: empty arrays, `false` flags, `0` counters, empty strings for
  attribution.

A future partial record should cost a badge, not a listing. And re-validate on read even for
data you wrote yourself — if it passed through anywhere editable, it is input again. Re-apply
the art-origin allowlist at read time, not only at write time.

## Make the sync idempotent, and prove it

A sync script you cannot re-run safely is a script nobody runs.

- **Byte-stable output.** No pretty-printing, deterministic key order, deterministic record
  sort. Re-running over unchanged upstream data must produce a byte-identical file — that is
  what makes the diff reviewable and "nothing changed" verifiable.
- **Deterministic sort comparators.** Collector numbers are strings and often non-numeric
  (`"TG01"`, `"SV01"`, `"Q1"`). Sort the numeric run first, then the lettered promos
  alphabetically. An unstable sort produces a diff on every run and makes the data
  unreviewable.
- **A dry run that writes nothing** and prints what it would have written.
- **No-op detection.** Compare the newly projected record against the stored one and skip
  identical writes. A run over an unchanged catalogue should report zero writes and say so.
- **Validate everything before writing anything.** A partial write ships an incomplete
  snapshot that still renders as if complete.
- **A plausibility floor.** Abort when the record count falls below a stated minimum. A
  truncated download is indistinguishable from a genuine catalogue shrink without this guard,
  and committing one is silent data loss.

Full annotated template: [references/sync-script-blueprint.md](references/sync-script-blueprint.md)
and the runnable [scripts/sync-skeleton.mjs](scripts/sync-skeleton.mjs) — start with
`node scripts/sync-skeleton.mjs --dry-run`, which fetches, validates and reports without writing.

## When you genuinely must fetch at request time

Prices, stock, anything user-specific — plus the transition period before a snapshot exists.
The machinery that makes a flaky upstream survivable, and the reasoning behind each piece, is
in [references/runtime-resilience.md](references/runtime-resilience.md).

The two rules that matter most:

- **Never cache a failure.** Cache successes only, so a card added upstream resolves on the
  next request without a manual invalidation.
- **Bound the retry rate separately from the cache.** Not caching failures means an outage
  re-asks on every render. Those are two different problems and they need two different
  mechanisms.

## Card art is its own problem

Orientation, rotation, origin allowlists, derive-versus-read, and whether to proxy, re-host or
hotlink: [references/card-art-and-images.md](references/card-art-and-images.md).

The one thing to internalise here: **never interpolate an id or a URL from your data into an
outbound fetch without an origin allowlist**. Card ids routinely come from editable
configuration, and an unchecked URL in a fetch is a server-side request forgery.

## Checklist before calling an ingestion done

- [ ] The source's field contract was measured, not read, and is written down with its date.
- [ ] Every field the census showed below 100% presence is typed optional.
- [ ] Facet vocabularies are derived from the fetched data, never hardcoded.
- [ ] The projection names what it drops and why.
- [ ] The snapshot carries a version; the reader rejects unknown versions.
- [ ] Re-running the sync over unchanged data produces a byte-identical result.
- [ ] A dry run exists and writes nothing.
- [ ] A record count floor aborts a truncated download before it overwrites anything.
- [ ] Nothing is written until everything validates.
- [ ] Generated data is committed, and the directory is not gitignored.
- [ ] A README beside the generated files says they are generated and names the command.
- [ ] Prices, if present, are on a separate cadence from identity.
- [ ] Every outbound art fetch passes an origin allowlist.
- [ ] The upstream terms and attribution are recorded next to the cached data.
