#!/usr/bin/env node
/**
 * @fileoverview Turns a downloaded JSON corpus into the evidence needed to type it correctly.
 *
 * Run by hand against a corpus on disk, before declaring types or building facets from it. It
 * separates "the docs list 40 fields" from "7 are always present and 11 appear under 5% of the
 * time" — a type that holds versus one that crashes on card 2000. Any field below 100% presence is
 * optional, whatever the docs imply.
 *
 * Reports the record count first (how a truncated download is later detected), then per-field
 * presence, scalar value vocabularies as facet candidates, one-level-deep sub-key presence for
 * nested fields, and the distinct hosts behind URL-valued fields.
 *
 * This file resolves the corpus and renders the report; the statistics themselves are computed by
 * the sibling module `./census-stats.mjs`.
 *
 * Zero dependencies. Node 20+, `node:` builtins only.
 */

import { readFileSync } from "node:fs";

import { census, isPlainObject, typeName } from "./census-stats.mjs";

/** @typedef {import("./census-stats.mjs").FieldStat} FieldStat */

/** Distinct scalar values a field may hold and still get a printed vocabulary. */
const DEFAULT_MAX_VALUES = 40;
/** Vocabulary entries printed per field before the remainder is counted as elided. */
const DEFAULT_TOP = 12;

const USAGE =
  "usage: field-census.mjs <file.json> [--at <key|dotted.path>] [--top <n>] [--max-values <n>]";

/**
 * Prints an error and exits with the code that classifies it.
 * @param {number} code - 2 for a bad invocation, 1 for a corpus that cannot be censused; the split
 *   tells a caller whether to fix the command or the data.
 * @param {string} message - What blocked the census, with the detail needed to act on it.
 * @returns {never} Never returns; code 2 also prints {@link USAGE}.
 */
function fail(code, message) {
  console.error(code === 2 ? `error: ${message}\n${USAGE}` : `error: ${message}`);
  process.exit(code);
}

/**
 * Parses argv into census options.
 * @param {string[]} argv - Arguments following the script path.
 * @returns {{file: string, at: string|null, top: number, maxValues: number}} `at` is a dotted path
 *   into the root when the records are not the root itself.
 * @throws Never; a malformed invocation exits 2.
 */
function parseArgs(argv) {
  let [file, at, top, maxValues] = [null, null, DEFAULT_TOP, DEFAULT_MAX_VALUES];
  const count = (raw, flag) =>
    /^\d+$/.test(raw ?? "") && Number(raw) > 0 ? Number(raw) : fail(2, `${flag} needs a positive integer`);

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--at") at = argv[++i] ?? fail(2, "--at requires a key or dotted path");
    else if (arg === "--top") top = count(argv[++i], "--top");
    else if (arg === "--max-values") maxValues = count(argv[++i], "--max-values");
    else if (arg.startsWith("-")) fail(2, `unknown option ${arg}`);
    else if (file === null) file = arg;
    else fail(2, "only one file may be censused per run");
  }
  if (file === null) fail(2, "a json file is required");
  return { file, at, top, maxValues };
}

/**
 * Reads and parses the corpus file.
 * @param {string} file - Path to a JSON document.
 * @returns {unknown} The parsed root value.
 * @throws Never; an unreadable file or a parse failure exits 1, the parse failure reporting the byte
 *   offset that locates a truncated or doubly-concatenated download.
 */
function loadCorpus(file) {
  let text;
  try {
    text = readFileSync(file, "utf8");
  } catch (err) {
    fail(1, `cannot read ${file}: ${err.message}`);
  }
  try {
    return JSON.parse(text);
  } catch (err) {
    // An unexpected end of input carries no position, yet the offset that matters is the file's own
    // end: compared against the length the source advertised, it confirms a truncated download.
    const pos = /position (\d+)/.exec(err.message)?.[1] ?? `${Buffer.byteLength(text)} (end of file)`;
    fail(1, `${file} is not valid JSON at byte offset ${pos}: ${err.message}`);
  }
}

/**
 * Asserts that every element of a candidate record array is a plain object.
 * @param {unknown[]} list - Candidate records.
 * @param {string} label - How the list was located, quoted back in the error.
 * @returns {object[]} The same list, once every element qualifies.
 * @throws Never; exits 1 naming the first offending index, since a scalar among records means the
 *   wrong path was chosen, not that the data is irregular.
 */
function requireObjects(list, label) {
  const bad = list.findIndex((r) => !isPlainObject(r));
  if (bad !== -1) fail(1, `${label} element ${bad} is ${typeName(list[bad])}, not an object`);
  return list;
}

/**
 * Locates the record array inside the corpus.
 * @param {unknown} root - Parsed root value.
 * @param {string|null} at - Dotted path to the records, or null to infer them.
 * @returns {{records: object[], source: string}} `source` names where the records were found, so the
 *   report states the interpretation it used instead of leaving it implied.
 * @throws Never; exits 1 when no record array can be identified, naming the root's top-level keys so
 *   the caller can pick one for `--at`.
 */
function resolveRecords(root, at) {
  if (at !== null) {
    let node = root;
    for (const segment of at.split(".")) {
      if (!isPlainObject(node) || !(segment in node)) fail(1, `--at "${at}" stops at "${segment}"`);
      node = node[segment];
    }
    if (!Array.isArray(node)) fail(1, `--at "${at}" is ${typeName(node)}, not an array`);
    return { records: requireObjects(node, `--at ${at}`), source: `--at ${at}` };
  }
  if (Array.isArray(root)) return { records: requireObjects(root, "root array"), source: "root array" };
  if (isPlainObject(root)) {
    // An id-keyed map is a record collection whose keys are ids, so its values are the records.
    const values = Object.values(root);
    if (values.length > 0 && values.every(isPlainObject)) {
      return { records: values, source: "values of an id-keyed root object" };
    }
    fail(1, `cannot infer the record array; pass --at with one of: ${Object.keys(root).join(", ")}`);
  }
  fail(1, `root is ${typeName(root)}; a census needs an array or an object of records`);
}

/** Formats a count as a right-aligned percentage of `total`. */
function pct(count, total) {
  return `${((count / total) * 100).toFixed(1).padStart(5)}%`;
}

/**
 * Prints an indented `count [percentage] label` block for one tally.
 * @param {Map<unknown, number>} tally - Counter keyed by the thing being counted.
 * @param {number} limit - Entries printed before the remainder is summarised; `Infinity` prints all.
 * @param {number|null} total - Denominator for a percentage column, or null to omit that column.
 * @param {boolean} quote - Render keys as JSON, which keeps an empty string or stray whitespace
 *   visible in a value vocabulary.
 * @returns {void} Ordering is count descending then key ascending.
 */
function printTally(tally, limit, total, quote) {
  const sorted = [...tally].sort((a, b) => b[1] - a[1] || String(a[0]).localeCompare(String(b[0])));
  for (const [key, n] of sorted.slice(0, limit)) {
    const share = total === null ? "" : ` ${pct(n, total)}`;
    console.log(`      ${String(n).padStart(7)}${share}  ${quote ? JSON.stringify(key) : key}`);
  }
  if (sorted.length > limit) console.log(`      ... ${sorted.length - limit} more elided`);
}

/**
 * Summarises an array field's element-count spread and a nested field's sub-object total.
 * @param {FieldStat} stat - Statistics for one field.
 * @returns {string} A semicolon-joined summary, empty for a field that is neither.
 */
function describeNesting(stat) {
  const parts = [];
  if (stat.lengths.length > 0) {
    const mean = (stat.lengths.reduce((a, b) => a + b, 0) / stat.lengths.length).toFixed(2);
    parts.push(`length min ${Math.min(...stat.lengths)}, max ${Math.max(...stat.lengths)}, mean ${mean}`);
  }
  if (stat.subObjects > 0) parts.push(`${stat.subObjects} sub-objects`);
  return parts.join("; ");
}

/**
 * Prints every census section for one corpus.
 * @param {Map<string, FieldStat>} stats - Census output, already ordered by descending presence.
 * @param {number} total - Record count, the denominator for presence percentages.
 * @param {number} top - Vocabulary entries printed per field.
 * @returns {void} A section with no qualifying field is skipped entirely. Fields at 100% presence
 *   are marked `ALWAYS`; every other field is optional and must be typed optional.
 */
function report(stats, total, top) {
  console.log(`\nFIELD PRESENCE (${total} records)`);
  const width = Math.max(...[...stats.keys()].map((k) => k.length));
  for (const [field, s] of stats) {
    const flag = s.present === total ? "  ALWAYS" : "";
    const kinds = [...s.kinds].join("|");
    console.log(`  ${field.padEnd(width)} ${String(s.present).padStart(7)} ${pct(s.present, total)}  ${kinds}${flag}`);
  }

  const vocab = [...stats].filter(([, s]) => s.allScalar && !s.valuesAbandoned && s.values.size > 0);
  if (vocab.length > 0) console.log("\nVALUE VOCABULARIES (facet candidates)");
  for (const [field, s] of vocab) {
    console.log(`  ${field} - ${s.values.size} distinct`);
    printTally(s.values, top, null, true);
  }

  const nested = [...stats].filter(([, s]) => s.subObjects > 0 || s.lengths.length > 0);
  if (nested.length > 0) console.log("\nNESTED FIELDS (one level deep)");
  for (const [field, s] of nested) {
    console.log(`  ${field} - ${describeNesting(s)}`);
    printTally(s.subKeys, Infinity, s.subObjects, false);
  }

  // Hosts are collected at a field's own value and one level below it, so a field whose children
  // hold the URLs appears here too — an images object of art URLs being the case that matters.
  const urls = [...stats].filter(([, s]) => s.urlCount > 0);
  if (urls.length > 0) console.log("\nURL HOSTS (allowlist evidence)");
  for (const [field, s] of urls) {
    console.log(`  ${field} - ${s.urlCount} urls`);
    printTally(s.hosts, Infinity, null, false);
  }
}

/**
 * Censuses one corpus and prints every section.
 * @returns {number} Exit code 0; a data problem exits 1 and a usage error exits 2 before returning.
 */
function main() {
  const { file, at, top, maxValues } = parseArgs(process.argv.slice(2));
  const { records, source } = resolveRecords(loadCorpus(file), at);

  // The count leads: comparing it to a recorded count is how a truncated download is later caught.
  console.log(`records           ${records.length}\nsource            ${source}`);
  if (records.length === 0) fail(1, "no records to census");

  report(census(records, maxValues), records.length, top);
  console.log(
    "\nMeasured value vocabularies are documentation, not contracts: derive facets from the data you" +
      " fetched, never from a hardcoded list.");
  return 0;
}

process.exit(main());
