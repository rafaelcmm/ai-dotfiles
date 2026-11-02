#!/usr/bin/env node
/**
 * @fileoverview First contact with an unknown JSON HTTP source: reports what the source
 * actually does, so the caller never has to assume.
 *
 * Run by hand before writing integration code against a trading-card-game data source. Every
 * published fact about a fan API — required headers, envelope, redirect and caching behaviour — is
 * a dated measurement that drifts in both directions, so measuring beats recalling.
 *
 * Reports final status, content type and length, the conditional-request headers, whether the
 * status changes without an identifying User-Agent, and the payload's top-level shape. Redirects
 * are never followed: on a data endpoint a redirect means something other than the expected
 * service replied, and that is the finding. A body past the size ceiling is refused, not buffered.
 *
 * Zero dependencies. Node 20+, `node:` builtins and global `fetch` only.
 */

import { writeFile } from "node:fs/promises";

import { describeShape, escapeOneLine } from "./json-shape.mjs";

/** User-Agent the probe identifies with. Operators use it to tell a probe from an attack. */
const PROBE_UA = "tcg-source-probe/1.0 (+https://example.invalid)";
/** Request deadline in milliseconds when `--timeout` is absent. */
const DEFAULT_TIMEOUT_MS = 20_000;
/**
 * Byte ceiling for a response body. `AbortSignal.timeout` bounds wall-clock time and never bytes, so
 * a body delivered fast but without end is read to completion before any deadline could fire. First
 * contact with an unknown or hostile source is exactly the case where that bites. The value clears a
 * ~9.5 MB corpus like LorcanaJSON's `allCards.json`, which `--save` exists to fetch, many times over.
 */
const MAX_RESPONSE_BYTES = 64 * 1024 * 1024;

/** Characters of a non-JSON body shown, escaped to one line. */
const BODY_PREVIEW_CHARS = 200;
/** Column width for labelled report lines, sized to the longest label. */
const LABEL_WIDTH = 18;

const USAGE = "usage: probe-json-source.mjs <url> [--no-ua] [--timeout <ms>] [--save <file>]";

/**
 * Prints a usage error and terminates the process.
 * @param {string} message - What the caller got wrong, as one lower-case phrase.
 * @returns {never} Exits 2, so a wrapper can tell a bad invocation from a bad answer.
 */
function failUsage(message) {
  console.error(`error: ${message}`);
  console.error(USAGE);
  process.exit(2);
}

/**
 * Parses argv into probe options, rejecting anything unrecognised.
 * @param {string[]} argv - Arguments following the script path.
 * @returns {{url: string, identify: boolean, timeoutMs: number, savePath: string|null}}
 *   `identify` is false when `--no-ua` asked for the header-less request alone.
 * @throws Never; a malformed invocation exits 2, so a half-understood argument list never reaches
 *   the network.
 */
function parseArgs(argv) {
  let url = null;
  let identify = true;
  let timeoutMs = DEFAULT_TIMEOUT_MS;
  let savePath = null;

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--no-ua") {
      identify = false;
    } else if (arg === "--timeout") {
      const raw = argv[++i];
      // A non-numeric or zero timeout makes AbortSignal.timeout either throw or abort
      // instantly, and both failures read as faults in the source.
      if (!/^\d+$/.test(raw ?? "") || Number(raw) === 0) {
        failUsage("--timeout requires a positive integer count of milliseconds");
      }
      timeoutMs = Number(raw);
    } else if (arg === "--save") {
      savePath = argv[++i] ?? failUsage("--save requires a file path");
    } else if (arg.startsWith("-")) {
      failUsage(`unknown option ${arg}`);
    } else if (url === null) {
      url = arg;
    } else {
      failUsage("only one url may be probed per run");
    }
  }

  if (url === null) failUsage("a url is required");
  return { url, identify, timeoutMs, savePath };
}

/**
 * Validates a target URL and forces it onto TLS.
 * @param {string} raw - Candidate URL as typed by the caller.
 * @returns {{target: URL, upgraded: boolean}} `upgraded` is true when the probe rewrote a supplied
 *   `http:`, which the report discloses: the measurement then belongs to another scheme.
 * @throws Never; an unparseable URL or a non-http(s) scheme exits 2, since probing `file:` or
 *   `data:` measures nothing about a remote source.
 */
function resolveTarget(raw) {
  let target;
  try {
    target = new URL(raw);
  } catch {
    failUsage(`not a parseable url: ${raw}`);
  }
  if (target.protocol !== "http:" && target.protocol !== "https:") {
    failUsage(`scheme ${target.protocol} is not probeable; use http: or https:`);
  }
  const upgraded = target.protocol === "http:";
  if (upgraded) target.protocol = "https:";
  return { target, upgraded };
}

/**
 * Issues one probe request and reads its body to completion.
 * @param {URL} target - Endpoint to measure.
 * @param {boolean} identify - Whether to send {@link PROBE_UA}. When false the header goes out
 *   empty, the closest the runtime gets to none: Node always transmits some User-Agent.
 * @param {number} timeoutMs - Deadline covering connect, headers and body.
 * @returns {Promise<{status: number, headers: Headers, body: string|null, bytes: number|null,
 *   oversize: string|null}>} `oversize` is a ready-to-print refusal naming both the offending size
 *   and the ceiling; when it is set `body` is null and no payload may be used. `bytes` is the
 *   measured length, null only when the body was never read.
 * @throws {Error} Propagates transport failures (DNS, TLS, reset, timeout) as source conditions.
 */
async function probe(target, identify, timeoutMs) {
  const response = await fetch(target, {
    headers: { Accept: "application/json", "User-Agent": identify ? PROBE_UA : "" },
    // Manual redirect handling is the deliverable, not an optimisation: a followed redirect
    // reports the status of whatever answered last and hides the hop.
    redirect: "manual",
    signal: AbortSignal.timeout(timeoutMs),
  });
  const result = { status: response.status, headers: response.headers, body: null, bytes: null,
    oversize: null };

  const declared = Number(response.headers.get("content-length"));
  if (Number.isInteger(declared) && declared > MAX_RESPONSE_BYTES) {
    result.oversize = `declared ${declared} bytes over the ${MAX_RESPONSE_BYTES}-byte ceiling; body not read`;
    // Drop the unread body so the socket is released instead of left draining.
    response.body?.cancel().catch(() => {});
    return result;
  }

  const body = await response.text();
  result.bytes = Buffer.byteLength(body);
  // content-length is advisory: absent or untrue is routine, so the real length is what decides.
  if (result.bytes > MAX_RESPONSE_BYTES) {
    result.oversize = `measured ${result.bytes} bytes over the ${MAX_RESPONSE_BYTES}-byte ceiling`;
    return result;
  }
  result.body = body;
  return result;
}

/** Prints one aligned `label: value` report line. */
function say(label, value) {
  console.log(`${`${label}:`.padEnd(LABEL_WIDTH)} ${value}`);
}

/**
 * Reports status, content and conditional-request headers for a response.
 * @param {{status: number, headers: Headers, bytes: number|null}} result - A completed probe.
 * @returns {void} Prints only. `etag` and `last-modified` are called out because they make a
 *   conditional request possible, and a multi-megabyte corpus belongs behind one.
 */
function reportHeaders(result) {
  say("status", result.status);
  say("content-type", result.headers.get("content-type") ?? "(absent)");
  const declared = result.headers.get("content-length");
  const measured = result.bytes === null ? "(absent)" : `(absent) - ${result.bytes} bytes measured`;
  say("content-length", declared ?? measured);
  for (const header of ["etag", "last-modified"]) {
    const value = result.headers.get(header);
    if (value !== null) say(header, `${value}  (conditional requests supported)`);
  }
}

/**
 * Measures whether the source's status depends on an identifying User-Agent.
 * @param {URL} target - The endpoint the primary request already measured.
 * @param {number} primaryStatus - Status the identified request received, compared against.
 * @param {number} timeoutMs - Deadline for this comparison request.
 * @returns {Promise<void>} Prints the comparison. A failure here is reported and swallowed: a
 *   flaky second request must not discard the primary measurement.
 */
async function reportUaSensitivity(target, primaryStatus, timeoutMs) {
  let line;
  try {
    const { status } = await probe(target, false, timeoutMs);
    const verdict = status === primaryStatus
      ? "same as with a User-Agent; this source does not gate on it today"
      : `DIFFERS from ${primaryStatus}; this source gates on User-Agent`;
    line = `${status} - ${verdict}`;
  } catch (err) {
    line = `request failed (${err.message}) - enforcement undetermined`;
  }
  say("status without ua", line);
  console.log("    (Node always sends some User-Agent; the probe sent an empty value)");
}

/**
 * Probes one source and reports every measurable fact about it.
 * @returns {Promise<number>} Exit code: 0 when the primary request returned 2xx, 1 for any other
 *   status or transport failure. Usage errors exit 2 before reaching here.
 */
async function main() {
  const { url, identify, timeoutMs, savePath } = parseArgs(process.argv.slice(2));
  const { target, upgraded } = resolveTarget(url);

  say("url", target.href);
  if (upgraded) say("note", "scheme upgraded from http: to https: before probing");
  say("sent ua", identify ? PROBE_UA : "(empty - --no-ua)");

  let result;
  try {
    result = await probe(target, identify, timeoutMs);
  } catch (err) {
    // A transport failure is a finding about the source, not a usage error, so it shares
    // the exit code of a bad status rather than the usage code.
    console.error(`request failed: ${err.message}`);
    return 1;
  }

  reportHeaders(result);

  if (result.oversize !== null) {
    say("refused", result.oversize);
    return 1;
  }

  if (result.status >= 300 && result.status < 400) {
    say("location", result.headers.get("location") ?? "(absent)");
    console.log(
      "\nStopped: redirects are not followed. On a data endpoint a redirect means something other" +
        " than the expected service is replying - a moved host, a captive portal, a login wall.\n" +
        "Probe the location directly before trusting it.",
    );
    return 1;
  }

  // Meaningful only when the primary request carried a User-Agent; under --no-ua the
  // header-less request already is the primary one.
  if (identify) await reportUaSensitivity(target, result.status, timeoutMs);

  let parsed;
  try {
    parsed = JSON.parse(result.body);
  } catch (err) {
    say("json", `does not parse (${err.message})`);
    say("body preview", escapeOneLine(result.body, BODY_PREVIEW_CHARS));
  }
  if (parsed !== undefined) {
    const [summary, ...detail] = describeShape(parsed);
    say("json shape", summary);
    for (const line of detail) console.log(line);
  }

  if (savePath !== null) {
    await writeFile(savePath, result.body);
    say("saved", `${savePath} - census it with: node scripts/field-census.mjs ${savePath}`);
  }

  return result.status >= 200 && result.status < 300 ? 0 : 1;
}

process.exit(await main());
