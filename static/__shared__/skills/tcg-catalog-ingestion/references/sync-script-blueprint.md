# Sync script blueprint

The shape of a card-data sync script that survives being re-run, being interrupted, and being
read six months later by someone else. Each guard carries the failure it prevents, because a
guard whose reason is unstated is the first one a later reader deletes.

## Contents

- [What a sync script is and is not](#what-a-sync-script-is-and-is-not)
- [Zero dependencies, plain ESM](#zero-dependencies-plain-esm)
- [File split](#file-split)
- [Transport](#transport)
- [Concurrency](#concurrency)
- [The guards, in order](#the-guards-in-order)
- [Projection](#projection)
- [Byte-stable output](#byte-stable-output)
- [Flags](#flags)
- [The report](#the-report)
- [Exit codes](#exit-codes)
- [Credentials, when the destination needs them](#credentials-when-the-destination-needs-them)
- [What to test](#what-to-test)
- [The README beside the data](#the-readme-beside-the-data)

## What a sync script is and is not

It **is** a developer-run batch job that reads an upstream source, projects it, and writes a
committed artefact. It runs on a laptop or in a deliberate CI step.

It is **not** a cron job, a webhook handler, a runtime module, or part of any request path.
That distinction buys you things worth protecting: generous timeouts, slow sequential passes,
loud failure, and no caching machinery at all. A script run is a single pass.

Say this in the file's own header. The next person to read it will otherwise try to "optimise"
a one-minute batch job that nobody is waiting on, or add a cache to something that needs none.

## Zero dependencies, plain ESM

Write it as plain `.mjs` with only `node:` builtins and global `fetch`.

- It never ships to a browser or a server runtime, so there is nothing to gain from a build
  step and a TypeScript toolchain to keep working.
- Node's own `--env-file` reads a `.env` without a dotenv dependency.
- A script with no dependencies still runs in two years.

The cost is that it cannot import your typed modules. Accept the duplication and make it
greppable: duplicate a version constant or a validation regex with a comment naming the file
that owns it, rather than introducing a build step to avoid two literals.

## File split

A sync script that reads, resolves, projects, writes and reports is doing five things. Split
it from the start rather than growing one file:

```
scripts/
  sync-<game>.mjs          CLI entry: arg parsing, orchestration, report
  <game>/
    source.mjs             upstream fetch + response narrowing
    projection.mjs         record -> stored shape
    destination.mjs        reading existing state + writing
```

Each file gets a header stating its single responsibility. The projection module is the one
worth isolating hardest: it is pure, it is the contract between writer and reader, and it is
the only part worth unit-testing heavily.

`scripts/sync-skeleton.mjs` carries the minimum version of that split — the entry point plus
`sync-projection.mjs` — because it covers one source with one entity pair. Separate the source
and destination modules out at the second source, the second destination, or 300 lines in the
entry point, whichever arrives first.

## Transport

```js
const TIMEOUT_MS = 45_000;            // generous: nothing is waiting on a batch job
const BACKOFF_MS = [2_000, 8_000];    // a schedule, not a base to double

async function fetchWithRetry(url, attempts = 3) {
  for (let attempt = 0; attempt < attempts; attempt++) {
    try {
      const response = await fetch(url, {
        headers: { "User-Agent": USER_AGENT, Accept: "application/json" },
        redirect: "manual",
        signal: AbortSignal.timeout(TIMEOUT_MS),
      });
      if (response.status >= 300 && response.status < 400) {
        throw new Error(`${url} redirected (HTTP ${response.status})`);
      }
      if (!response.ok) {
        const error = new Error(`${url} -> HTTP ${response.status}`);
        error.status = response.status;
        throw error;        // stamped, so the catch can tell status from transport
      }
      return await response.json();
    } catch (err) {
      if (err.status !== undefined || attempt === attempts - 1) throw err;
      await sleep(BACKOFF_MS[attempt] ?? 8000);
    }
  }
}
```

Four decisions encoded there, each worth keeping:

- **Retry transport failures, never HTTP statuses.** A non-2xx is a stable answer from a
  reachable server. Re-asking spends someone else's bandwidth to hear it again. Stamping the
  error with `status` is what lets the catch distinguish the two.
- **Three attempts with growing backoff, not one retry.** A single back-to-back retry fails
  against a slow burst, because both attempts land inside it. Two seconds then eight spreads
  the attempts across a window wide enough to clear the bursts actually observed on free APIs.
- **Generous timeouts.** Nothing is waiting on this. A run that takes thirty seconds longer
  beats a run that dies and has to be restarted by hand.
- **Refuse redirects.** These endpoints answer directly, so a 3xx means something other than
  the expected service is replying — and a followed redirect is an unvalidated outbound
  request from whatever machine runs the sync.

Send a `User-Agent` naming the project with a contact URL, on every request, whether or not the
source currently enforces it.

## Concurrency

```js
async function mapPooled(items, worker, limit = 4) {
  const results = new Array(items.length);
  let cursor = 0;
  await Promise.all(
    Array.from({ length: Math.min(limit, items.length) }, async () => {
      while (cursor < items.length) {
        const index = cursor++;
        results[index] = await worker(items[index], index);
      }
    })
  );
  return results;
}
```

Preserve input order — a sync whose output order depends on response timing is not idempotent.

**Default the limit to four and comment that it is a courtesy ceiling, not a performance
target.** Free card APIs begin throttling a single IP somewhere around four to eight concurrent
requests. There is nothing to win by raising it and a shared free service to lose. If a
sequential pass over a per-card endpoint takes two minutes, that is fine; it is a batch job.

## The guards, in order

Run them in this order, and abort the whole run on any of them. Use a distinct error class
(`SyncAborted`) so the exit path can print "ABORTED — nothing was written" rather than a stack
trace that looks like a crash.

`scripts/sync-skeleton.mjs` implements guards 1 and 3, which need nothing from a source beyond
what every source has. Guards 2, 4 and 5 depend on what the source declares and what the
destination accepts, so they are yours to add — guard 4 especially, since the reject-versus-default
split the skill builds on rests entirely on it.

1. **Version gate.** Fetch the small metadata endpoint first. Abort on a major-version change:
   the field contract may have changed shape and a human must re-read the docs and redo the
   census. Print the version either way, so a minor bump is visible in the log.
2. **Declared-count check.** When the source states how many records a set or page holds,
   compare it against what arrived. Fewer means a truncated download.
3. **Plausibility floor.** Abort when the total record count is below a stated minimum. Write
   the reasoning into the constant: the floor exists because a truncated download or an
   upstream error page is otherwise indistinguishable from a genuine catalogue shrink, and
   committing one is silent data loss.
4. **Required-field check on every record.** A record missing a render-critical field is a
   contract change. List the offending ids explicitly rather than counting them — a silent
   exclusion is the exact failure these guards exist to prevent.
5. **Size ceiling**, when the destination has one. Fail the record rather than truncating it.

Then, and only then, write. **Fetch and validate everything before writing anything.**

## Projection

```js
/**
 * Fields kept from an upstream record. Everything else is deliberately dropped:
 * <list them>, none of which any surface renders.
 *
 * Keeping this list narrow is what holds the committed index to ~3 MB rather
 * than the source file's ~9.5 MB.
 */
const PROJECTED_FIELDS = ["id", "name", /* ... */];

function project(raw) {
  const out = {};
  for (const field of PROJECTED_FIELDS) {
    if (Object.hasOwn(raw, field)) out[field] = raw[field];
  }
  return out;
}
```

- **Omit absent keys entirely.** Never write `null` for a field the source did not send. That
  omission is most of the size saving, and it lets the reader treat "key present" and "value
  known" as the same question.
- **Name the dropped fields and why**, once, in the constant's docblock. Otherwise someone
  re-adds the excluded field without knowing the exclusion was deliberate.
- Project nested objects explicitly too — keeping two of six image keys rather than the whole
  object is often the largest single win.

When building the output map, prefer `Object.create(null)` over `{}` if the keys come from
upstream data. A key of `__proto__` on a plain object reassigns the prototype through the
inherited accessor and silently drops that record.

## Byte-stable output

```js
await writeFile(path, `${JSON.stringify(value)}\n`, "utf8");
```

- `JSON.stringify` with no indentation. Pretty-printing roughly doubles the size for no reader
  benefit.
- Deterministic insertion order for object keys, deterministic sort for arrays.
- Integer-index-like keys are serialised in ascending numeric order whatever the insertion
  order. That is stable across runs, and it **overrides whatever sort you applied** — so when
  the sort order is what makes the diff readable, serialise the map by hand from ordered
  entries rather than relying on insertion order.

**Collector-number sorting needs a real comparator.** Collector numbers are strings and often
non-numeric:

```js
function byCollectorNumber(a, b) {
  const na = Number.parseInt(a.localId, 10);
  const nb = Number.parseInt(b.localId, 10);
  const aNum = Number.isFinite(na) && /^\d/.test(a.localId);
  const bNum = Number.isFinite(nb) && /^\d/.test(b.localId);
  if (aNum && bNum && na !== nb) return na - nb;
  if (aNum && !bNum) return -1;
  if (!aNum && bNum) return 1;
  // Code-unit comparison, never localeCompare: ICU collation varies by machine, so the
  // committed diff would depend on who ran the sync.
  return a.localId < b.localId ? -1 : a.localId > b.localId ? 1 : 0;
}
```

Numeric run first, lettered promos after, alphabetical among themselves — the order a collector
flips through a binder. An unstable sort produces a diff on every run, which makes the
committed data unreviewable and hides the real changes.

**Prove the stability rather than asserting it.** Run the sync twice and compare checksums. It
is a one-line check and it is the acceptance criterion for the whole script.

## Flags

| Flag | Behaviour |
|---|---|
| *(none)* | sync records that are missing, a different version, or byte-different |
| `--dry-run` | resolve, project, report, **write nothing** |
| `--check` | report-only: list missing, stale and unresolvable records; non-zero exit if any are missing |
| `--force` | rewrite every record even when byte-identical (after a version bump) |
| `--only <value>` | restrict to one record, for debugging a single awkward card |
| `--out <dir>` | destination, so a scratch run cannot overwrite committed data |

`--dry-run` and `--out` are implemented in `scripts/sync-skeleton.mjs`; `--check`, `--force`
and `--only` are specified here and are what to add as the script grows past a first ingest.

`--check` is the pre-flight gate before anything starts reading the snapshot. Flipping readers
over to a snapshot while records are unsynced silently downgrades exactly those records.

## The report

One line per non-OK record, then a summary:

```
Sync — 128 records
  updated      12
  unchanged   114
  not found     1   (unl-999-219)
  failed        1   (write rejected: ... )
```

- **Report the no-op count.** "unchanged 114" is the idempotency claim being demonstrated on
  every run.
- **Name the failures individually.** A count tells the operator something is wrong; an id
  tells them what to fix.
- **Never print secrets or full payloads** outside `--dry-run`.
- When a cap or a sample bounded the run, say what was dropped. Silent truncation reads as
  complete coverage.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | everything resolved; writes may legitimately be zero |
| 1 | any record unresolved, or any write failed |
| 2 | usage error |

A guard abort is exit 1 with `ABORTED` and an explicit "no files were written".

## Credentials, when the destination needs them

The upstream card source is usually public. The destination may not be.

- Credentials come from the environment or a credentials file, **never from argv** — arguments
  are visible in process listings and shell history.
- Validate the destination host against an explicit pattern before sending anything. A
  mistyped or injected host is a credential leak.
- Cache a fetched token in-process for the run; a script run is short.
- Die loudly on a missing variable, naming the variable, in one line.
- No secret reaches a logger, including inside an error message.
- Make identifiers for the destination explicit arguments rather than ambient environment
  values, so a stale shell cannot point a write at the wrong target.

## What to test

Unit-test the pure parts. Do not unit-test the network.

- **The projection**, including the awkward records: a missing optional field, an unknown
  nested shape, the stable key order.
- **The round trip**: a record produced by the projection parses cleanly through the reader's
  parser. This is the contract lock between writer and reader and it is the most valuable test
  in the whole script — it is the one that fails when the two drift.
- **The sort comparator**, with non-numeric collector numbers.
- **The guards**: a short count aborts, a missing required field aborts, an oversized record
  fails that record.

The network paths are covered by `--dry-run` against the live source, which is a better test
than a mock because it also catches contract drift.

## The README beside the data

Three short sections in the generated-data directory:

1. These files are generated — never hand-edit them.
2. Run `<command>` when a new set releases or the upstream bumps its format version, then
   commit the diff.
3. The current upstream version and sync timestamp, and where the field contract is documented.

Add a line to the script's own header saying the generated files **are** the runtime data
source and must be committed. Someone will otherwise gitignore a `data/` directory out of
habit and break the build in a way that looks like a missing dependency.
