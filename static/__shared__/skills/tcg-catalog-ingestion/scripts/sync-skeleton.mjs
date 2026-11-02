#!/usr/bin/env node
/**
 * @fileoverview Fetches a trading-card-game catalogue from an upstream source and writes a
 * committed, projected snapshot to disk — no transformation beyond field projection, no serving.
 * Developer-run only and never on a request path: the application reads the generated files, so an
 * upstream outage cannot reach users, and those files are the runtime data source.
 * Transport, guards, orchestration and exit codes live here; the stored shape and its ordering are
 * the contract in `./sync-projection.mjs`.
 * Copyable template: every site needing an edit for another source is marked `// ADAPT:`, and as
 * shipped it runs against LorcanaJSON, which is public, unauthenticated and static.
 *
 * Usage: node sync-skeleton.mjs [--dry-run] [--out <dir>]
 */

import { mkdir, readFile, writeFile } from "node:fs/promises";
import { setTimeout as delay } from "node:timers/promises";
import { join, resolve } from "node:path";
import {
  CARD_FIELDS, SET_FIELDS, SNAPSHOT_VERSION, compareCards, compareCodes, project,
  serializeOrderedMap,
} from "./sync-projection.mjs";

// ADAPT: your project and a contact URL. An operator of a free service seeing unusual traffic can
// then ask who is responsible instead of blocking an anonymous client.
const USER_AGENT = "tcg-catalog-sync/1.0 (+https://example.com/contact)";
// ADAPT: upstream endpoints. METADATA_URL must be small — it is fetched ahead of the corpus purely
// to decide whether the corpus is safe to trust.
const METADATA_URL = "https://lorcanajson.org/files/current/en/metadata.json";
const CORPUS_URL = "https://lorcanajson.org/files/current/en/allCards.json";
// ADAPT: the upstream major this projection was written against. A major bump means the field
// contract changed shape and a human must re-read the upstream docs before any data is trusted.
// Minor bumps have been additive in practice, which is a track record and not a guarantee.
const SUPPORTED_FORMAT_PREFIX = "2.";
// ADAPT: plausibility floor, below the smallest catalogue you would believe. Without it a truncated
// download or an error page is indistinguishable from a genuine shrink and overwrites a good
// snapshot.
const MIN_EXPECTED_CARDS = 3000;
// Largest response body this sync will read, counted after any decompression. A timeout bounds
// wall-clock time, never bytes: a body delivered quickly but without end is read to completion
// before any timeout could fire, and a small compressed payload can expand without limit. The
// ceiling sits well above the real corpus (~9.5 MB), so a genuine catalogue can grow for years.
// ADAPT: raise only against a measured corpus size, never to make a surprise response fit.
const MAX_RESPONSE_BYTES = 64 * 1024 * 1024;
// The retry budget comes from measurement, not taste. Response times on these free sources are
// uneven, with slow bursts: a 45-second per-request deadline and three attempts spread two seconds
// then eight cleared every burst observed, while a back-to-back retry did not, because a slow burst
// outlasts two attempts that both land inside it. BACKOFF_MS holds the waits between attempts, so
// it carries one entry fewer than MAX_ATTEMPTS.
const REQUEST_TIMEOUT_MS = 45_000;
const MAX_ATTEMPTS = 3;
const BACKOFF_MS = [2_000, 8_000];
// Concurrent request ceiling for `mapPooled`: a courtesy ceiling toward an unmetered free service,
// not a performance target. Raising it moves cost onto someone else's bandwidth to save seconds.
const CONCURRENCY = 4;

/**
 * Signals that a guard refused the run: upstream data was present but not trustworthy. Distinct from
 * a crash so the exit path can promise nothing was written — every guard throws before the first
 * write, which is what makes that promise true.
 */
class SyncAborted extends Error {
  name = "SyncAborted";
}

/**
 * Reads the two supported flags, rejecting anything else rather than guessing.
 * @param {string[]} argv - Arguments after the script path; `--dry-run` and `--out <dir>`.
 * @returns {{dryRun: boolean, outDir: string}} `outDir` is absolute, so later joins cannot depend on
 *   the working directory.
 * @throws {SyncAborted} On an unknown flag or valueless `--out`: falling back to the default would
 *   silently write the snapshot somewhere the operator did not ask for.
 */
function parseArgs(argv) {
  let dryRun = false;
  let outDir = "./data"; // ADAPT: default snapshot directory, relative to the invocation.
  for (let i = 0; i < argv.length; i += 1) {
    if (argv[i] === "--dry-run") dryRun = true;
    else if (argv[i] === "--out") {
      outDir = argv[i + 1];
      if (!outDir || outDir.startsWith("--")) throw new SyncAborted("--out requires a directory");
      i += 1;
    } else throw new SyncAborted(`unknown argument: ${argv[i]}`);
  }
  return { dryRun, outDir: resolve(outDir) };
}

/**
 * Reads a response body, abandoning the transfer once it passes `MAX_RESPONSE_BYTES`.
 *
 * Counting while streaming is the point: a ceiling checked after the body is in memory has already
 * paid the cost it exists to refuse. The count is of decoded bytes, which is the figure that bounds
 * memory and so bounds a compressed payload that expands without limit.
 * @param {Response} response - A 2xx response whose body has not been consumed.
 * @param {string} url - Fetched URL, named in the refusal so the operator knows which endpoint.
 * @returns {Promise<string>} The body decoded as UTF-8.
 * @throws {SyncAborted} Once the running total passes the ceiling, cancelling the stream first so
 *   the connection is not left draining in the background.
 */
async function readCappedBody(response, url) {
  const reader = response.body.getReader();
  const chunks = [];
  let total = 0;
  for (let chunk = await reader.read(); !chunk.done; chunk = await reader.read()) {
    total += chunk.value.byteLength;
    if (total > MAX_RESPONSE_BYTES) {
      await reader.cancel();
      throw new SyncAborted(`${url} exceeded the ${MAX_RESPONSE_BYTES}-byte ceiling`);
    }
    chunks.push(chunk.value);
  }
  return Buffer.concat(chunks).toString("utf8");
}

/**
 * Fetches a URL and parses its JSON body, retrying transport failures — timeout, DNS, dropped socket
 * — on the `BACKOFF_MS` schedule. A non-2xx status is never retried: it is a stable answer from a
 * reachable server, so re-asking spends the operator's time and upstream bandwidth for the same
 * reply.
 * @param {string} url - Absolute URL returning JSON.
 * @returns {Promise<unknown>} The parsed document.
 * @throws {SyncAborted} On a redirect, a body over the size ceiling, a non-2xx status, an exhausted
 *   attempt budget, or a non-JSON body — usually an HTML error page served with a 200, which must
 *   not be mistaken for data. Every message names the URL, since the diagnosis is "which endpoint,
 *   and why".
 */
async function fetchWithRetry(url) {
  let lastError;
  for (let attempt = 1; attempt <= MAX_ATTEMPTS; attempt += 1) {
    try {
      const response = await fetch(url, {
        headers: { "user-agent": USER_AGENT, accept: "application/json" },
        // These endpoints answer directly, so a 3xx means something other than the expected
        // service is replying. Following it would fetch the snapshot's contents from an
        // unvalidated host, and every guard below would still pass on a well-shaped forgery.
        redirect: "manual",
        signal: AbortSignal.timeout(REQUEST_TIMEOUT_MS),
      });
      if (response.status >= 300 && response.status < 400) {
        const target = response.headers.get("location") ?? "(no location header)";
        throw new SyncAborted(`${url} redirected to ${target} — refusing to follow`);
      }
      if (!response.ok) {
        throw new SyncAborted(`${url} returned HTTP ${response.status} ${response.statusText}`);
      }
      // An early-out for a body that announces itself as oversized, saving the transfer. It is
      // only an early-out: a compressed response carries no content-length once decoded, so this
      // check does not fire against most real sources and `readCappedBody` is the enforcement.
      const declared = Number.parseInt(response.headers.get("content-length") ?? "", 10);
      if (declared > MAX_RESPONSE_BYTES) {
        throw new SyncAborted(`${url} declares ${declared} bytes, over the ceiling`);
      }
      const body = await readCappedBody(response, url);
      try {
        return JSON.parse(body);
      } catch (error) {
        throw new SyncAborted(`${url} did not return JSON: ${error.message}`);
      }
    } catch (error) {
      if (error instanceof SyncAborted) throw error;
      lastError = error;
      if (attempt < MAX_ATTEMPTS) await delay(BACKOFF_MS[attempt - 1] ?? BACKOFF_MS.at(-1));
    }
  }
  throw new SyncAborted(`${url} unreachable after ${MAX_ATTEMPTS} tries: ${lastError?.message}`);
}

/**
 * Applies `worker` across `items` with at most `CONCURRENCY` in flight, preserving input order.
 * Unused against this upstream, which ships the whole catalogue in one file; kept because it is the
 * shape every per-record source needs — a fan-out over a per-card endpoint, the common case. Input
 * order survives whatever the completion order, so deterministic input stays deterministic.
 * @param {Array<T>} items - Work items, claimed left to right.
 * @param {(item: T, index: number) => Promise<R>} worker - Called exactly once per item.
 * @returns {Promise<R[]>} Results positionally aligned with `items`.
 * @throws Whatever `worker` throws; the first rejection propagates, abandoning in-flight work.
 * @template T, R
 */
async function mapPooled(items, worker) {
  const results = new Array(items.length);
  let next = 0;
  const lanes = Array.from({ length: Math.min(CONCURRENCY, items.length) }, async () => {
    while (next < items.length) {
      const index = next++;
      results[index] = await worker(items[index], index);
    }
  });
  await Promise.all(lanes);
  return results;
}

/**
 * Refuses the run when the upstream format version crosses a major boundary, printing it either way
 * so the operator sees which contract the snapshot was built against.
 * @param {unknown} metadata - Parsed metadata document.
 * @returns {{formatVersion: string, generatedOn: string|null}} Provenance for the sidecar file.
 * @throws {SyncAborted} When the version is missing or outside the supported major, whose bump
 *   silently invalidates the projection: the snapshot still parses and now means something else.
 */
function assertFormatVersion(metadata) {
  const formatVersion = metadata?.formatVersion;
  if (typeof formatVersion !== "string") throw new SyncAborted("metadata has no formatVersion");
  console.log(`upstream formatVersion: ${formatVersion}`);
  if (!formatVersion.startsWith(SUPPORTED_FORMAT_PREFIX)) {
    throw new SyncAborted(`formatVersion ${formatVersion} is outside supported `
      + `${SUPPORTED_FORMAT_PREFIX}x — re-read the upstream docs and update the projection first`);
  }
  return { formatVersion, generatedOn: metadata.generatedOn ?? null };
}

/**
 * Builds the provenance sidecar, reusing the previous sync timestamp when the data bytes match. A
 * run finding unchanged upstream data must produce no diff at all or every scheduled run commits
 * noise, and the timestamp is the only field that moves on its own.
 *
 * This file is where `SNAPSHOT_VERSION` reaches a reader, so it is the file a reader must consult
 * before trusting the data files beside it.
 * @param {string} outDir - Absolute snapshot directory, which need not exist yet.
 * @param {Record<string, string>} data - Data file contents keyed by file name.
 * @param {{formatVersion: string, generatedOn: string|null}} upstream - Version and build time.
 * @param {Record<string, number>} counts - Per-entity record counts.
 * @returns {Promise<string>} The sidecar text, newline-terminated.
 */
async function buildProvenance(outDir, data, upstream, counts) {
  const read = async (name) => readFile(join(outDir, name), "utf8").catch(() => "");
  const unchanged = (await Promise.all(Object.entries(data)
    .map(async ([name, text]) => (await read(name)) === text))).every(Boolean);
  let syncedAt = new Date().toISOString();
  const previous = unchanged ? await read("source.json") : "";
  if (previous) {
    try {
      syncedAt = JSON.parse(previous).syncedAt ?? syncedAt;
    } catch {
      // A corrupt sidecar is no reason to abort: it is regenerated here from fresh data.
    }
  }
  const sidecar = { snapshotVersion: SNAPSHOT_VERSION, upstream: { url: CORPUS_URL, ...upstream },
    syncedAt, counts };
  return `${JSON.stringify(sidecar, null, 2)}\n`; // Indented: a human reads this file directly.
}

/**
 * Fetches, validates, projects, and writes the snapshot. Ordering is the contract: everything is
 * fetched and every guard passes before the first byte is written, because a partial write ships an
 * incomplete snapshot that still parses and renders as if complete — worse than none.
 * @param {{dryRun: boolean, outDir: string}} options - `dryRun` runs every step but the writes.
 * @returns {Promise<void>} Resolves with the snapshot on disk, or after the dry-run report.
 * @throws {SyncAborted} From the version gate, the corpus shape check, or the plausibility floor.
 */
async function sync({ dryRun, outDir }) {
  const upstream = assertFormatVersion(await fetchWithRetry(METADATA_URL));
  console.log(`fetching ${CORPUS_URL}`);
  const corpus = await fetchWithRetry(CORPUS_URL);
  // ADAPT: the corpus shape. `cards` and `sets` are this upstream's top-level containers.
  const cards = corpus?.cards;
  const sets = corpus?.sets ?? {};
  if (!Array.isArray(cards)) throw new SyncAborted("corpus has no `cards` array");
  // ADAPT: guard 2 — check each set's cardCounts.total against arrivals; a shortfall is truncation.
  if (cards.length < MIN_EXPECTED_CARDS) {
    throw new SyncAborted(`only ${cards.length} cards, floor is ${MIN_EXPECTED_CARDS} — refusing `
      + "to overwrite the committed snapshot with a truncated download or an error page");
  }
  // ADAPT: the snapshot key. An upstream-stable id keeps consumer references valid across syncs.
  // ADAPT: guard 4 — reject a record missing any render-critical field, listing the offending ids.
  // Skipping it is what ships half-filled cards that a surface renders as though complete.
  const cardEntries = [...cards].sort(compareCards)
    .map((card) => [String(card.id), project(card, CARD_FIELDS)]);
  const setEntries = Object.keys(sets).sort(compareCodes)
    .map((code) => [code, project(sets[code], SET_FIELDS)]);
  const data = { "cards.json": serializeOrderedMap(cardEntries),
    "sets.json": serializeOrderedMap(setEntries) };
  const counts = { cards: cardEntries.length, sets: setEntries.length };
  const files = { ...data, "source.json": await buildProvenance(outDir, data, upstream, counts) };

  if (dryRun) {
    console.log(`DRY RUN — would write to ${outDir}:`);
  } else {
    await mkdir(outDir, { recursive: true });
    // ADAPT: guard 5 — where the destination caps a record, fail that record rather than truncate.
    for (const [name, text] of Object.entries(files)) await writeFile(join(outDir, name), text);
    console.log(`wrote ${outDir}:`);
  }
  for (const [name, text] of Object.entries(files)) {
    console.log(`  ${name.padEnd(12)} ${Buffer.byteLength(text).toLocaleString("en-US")} bytes`);
  }
  console.log(`cards: ${counts.cards}   sets: ${counts.sets}   `
    + `upstream ${upstream.formatVersion} generated ${upstream.generatedOn}`);
  console.log("These files are the runtime data source. Commit them; never gitignore them.");
}

try {
  await sync(parseArgs(process.argv.slice(2)));
} catch (error) {
  if (error instanceof SyncAborted) console.error(`ABORTED — nothing was written: ${error.message}`);
  else console.error(`UNEXPECTED FAILURE: ${error?.stack ?? error}`);
  process.exitCode = 1;
}
