/**
 * @fileoverview Pure per-field statistics over an array of JSON records: presence counts, value
 * vocabularies, one-level-deep sub-key counts, array length spreads and URL hosts.
 *
 * This layer only computes. It reads no files, parses no arguments, prints nothing and never exits,
 * so it can be exercised directly against a literal array of records. `field-census.mjs` is the
 * command-line tool that resolves a corpus and renders these statistics.
 *
 * Zero dependencies. Node 20+.
 */

/** Names a value's JSON shape in one word: `null`, `array`, `object`, or its typeof. */
export function typeName(value) {
  return value === null ? "null" : Array.isArray(value) ? "array" : typeof value;
}

/** Reports whether a value is a plain non-array object, the shape every record must have. */
export function isPlainObject(value) {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

/** Increments a counter keyed by `key` in `map`. */
function bump(map, key) {
  map.set(key, (map.get(key) ?? 0) + 1);
}

/**
 * Tallies one value against a field's URL hosts when it looks like a URL.
 * @param {FieldStat} stat - The field's accumulator, mutated in place.
 * @param {unknown} value - Candidate URL; anything but an `http`-prefixed string is ignored.
 * @returns {void} An `http`-prefixed string that will not parse is tallied under `(unparseable)`
 *   rather than dropped, because a malformed URL in a corpus is itself a finding.
 */
function noteUrl(stat, value) {
  if (typeof value !== "string" || !value.startsWith("http")) return;
  stat.urlCount += 1;
  try {
    bump(stat.hosts, new URL(value).host);
  } catch {
    bump(stat.hosts, "(unparseable)");
  }
}

/**
 * Per-field statistics accumulated over a record array.
 * @typedef {object} FieldStat
 * @property {number} present - Records holding this field at all. Against the total record count it
 *   gives the presence percentage, and anything below 100% must be typed optional.
 * @property {Set<string>} kinds - Every JSON shape observed for the value, as {@link typeName} words.
 *   More than one entry means the field is a union.
 * @property {boolean} allScalar - True while every observed value was a string, number or boolean.
 *   `null` clears it, so a nullable field never qualifies for a value vocabulary.
 * @property {Map<unknown, number>} values - Distinct scalar value to occurrence count. Empty means
 *   either nothing scalar was seen or the tally was abandoned; `valuesAbandoned` disambiguates.
 * @property {boolean} valuesAbandoned - True once distinct values exceeded the caller's ceiling. The
 *   field is free text rather than a facet, and the tally was dropped outright rather than left
 *   partial, so a stale half-vocabulary can never be mistaken for the real one.
 * @property {Map<string, number>} subKeys - Sub-key to the number of sub-objects carrying it, one
 *   level deep. `subObjects` is its denominator, not the record count.
 * @property {number} subObjects - Objects inspected one level down: the field's own object values
 *   plus the object elements of its array values.
 * @property {number[]} lengths - One element-count per array value, giving the min, max and mean
 *   spread. Empty for a field that is never an array.
 * @property {Map<string, number>} hosts - URL host to occurrence count, the evidence an origin
 *   allowlist is written from.
 * @property {number} urlCount - URL-looking strings seen for this field, counting those found one
 *   level below it; a non-zero count is what puts the field in the host report.
 */

/**
 * Creates an empty statistics accumulator for one field.
 * @returns {FieldStat} A mutable accumulator with every counter zeroed and every tally empty.
 */
export function newStat() {
  return { present: 0, kinds: new Set(), allScalar: true, values: new Map(), valuesAbandoned: false,
    subKeys: new Map(), subObjects: 0, lengths: [], hosts: new Map(), urlCount: 0 };
}

/**
 * Folds one field value into that field's statistics.
 * @param {FieldStat} stat - Accumulator for the field, mutated in place.
 * @param {unknown} value - The value this record holds for the field.
 * @param {number} maxValues - Distinct-value ceiling past which the vocabulary is abandoned.
 * @returns {void} Sub-keys and URLs are collected one level deep only, since the census exists to
 *   type the top level. `null` counts as non-scalar, so a nullable field earns no vocabulary.
 */
export function accumulate(stat, value, maxValues) {
  stat.present += 1;
  const kind = typeName(value);
  stat.kinds.add(kind);
  if (kind === "object") {
    stat.allScalar = false;
    stat.subObjects += 1;
    for (const [k, sub] of Object.entries(value)) {
      bump(stat.subKeys, k);
      noteUrl(stat, sub);
    }
    return;
  }
  if (kind === "array") {
    stat.allScalar = false;
    stat.lengths.push(value.length);
    for (const element of value) {
      noteUrl(stat, element);
      if (!isPlainObject(element)) continue;
      stat.subObjects += 1;
      for (const k of Object.keys(element)) bump(stat.subKeys, k);
    }
    return;
  }
  if (kind !== "string" && kind !== "number" && kind !== "boolean") {
    stat.allScalar = false;
    return;
  }
  noteUrl(stat, value);
  if (stat.valuesAbandoned) return;
  bump(stat.values, value);
  // Past the ceiling the field is free text, not a facet: drop the tally, never grow it per record.
  if (stat.values.size > maxValues) {
    stat.values.clear();
    stat.valuesAbandoned = true;
  }
}

/**
 * Builds the per-field census over every record.
 * @param {object[]} records - The record array; every element must be a plain object.
 * @param {number} maxValues - Distinct-value ceiling passed to {@link accumulate}.
 * @returns {Map<string, FieldStat>} Field name to statistics, ordered by descending presence then name,
 *   so always-present fields lead and runs stay comparable. A field absent from a record simply
 *   does not count toward its presence, which is what makes the percentages meaningful.
 */
export function census(records, maxValues) {
  const stats = new Map();
  for (const record of records) {
    for (const [field, value] of Object.entries(record)) {
      if (!stats.has(field)) stats.set(field, newStat());
      accumulate(stats.get(field), value, maxValues);
    }
  }
  return new Map([...stats].sort((a, b) => b[1].present - a[1].present || a[0].localeCompare(b[0])));
}
