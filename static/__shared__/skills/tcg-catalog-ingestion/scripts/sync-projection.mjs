/**
 * @fileoverview Defines the stored shape of a card snapshot: which upstream fields survive, the
 * order records and keys are written in, and the serialisation that makes the bytes stable. Pure
 * value transformation — no I/O, no network, no process exit — which is what makes this the layer
 * worth unit-testing heavily.
 *
 * This is the contract between the writer that produces a snapshot and every reader that consumes
 * one. Changing a field list, a sort, or a key order changes data that is already committed and
 * already being read, so it is a data migration rather than a refactor, and `SNAPSHOT_VERSION` is
 * how a reader notices.
 */

/**
 * Projection contract version, carried once in `source.json` and in no other output file.
 *
 * Bump it whenever a field list, a record sort, or a key order changes, so a reader can refuse a
 * snapshot it does not understand. Keeping it to one constant in one place is deliberate: a copy
 * stamped into every data file is a set of values that can disagree with each other, and the
 * disagreement is unresolvable after the fact.
 *
 * The obligation that buys falls on the reader. Read `source.json` first and treat the whole
 * snapshot as unresolvable when `snapshotVersion` is absent or unrecognised, rather than guessing:
 * a data file on its own carries no version to reject on, and a stale projection parses perfectly
 * while meaning something else.
 */
export const SNAPSHOT_VERSION = 1;
// ADAPT: exactly the fields the application reads, in output order. Everything else upstream ships
// is dropped on purpose — a snapshot mirroring upstream inherits every upstream change as a diff.
// Absent keys are omitted, not written as null; across sparse records that is most of the saving.
export const CARD_FIELDS = [
  "id", "setCode", "number", "fullName", "name", "version", "type", "rarity", "color", "cost",
  "inkwell", "strength", "willpower", "lore", "moveCost", "subtypes", "keywordAbilities",
  "fullText", "images",
];
// ADAPT: as CARD_FIELDS, for the secondary entity.
export const SET_FIELDS = ["name", "number", "type", "releaseDate", "hasAllCards", "cardCounts"];

/**
 * Copies the listed fields out of a record, in list order, omitting absent keys. List order becomes
 * the snapshot's key order, part of what keeps output byte-stable: insertion order survives
 * `JSON.stringify` for non-integer-like keys.
 *
 * Presence is `Object.hasOwn`, so an inherited key is never copied out of a record that arrived
 * carrying a prototype, and a key the source sent holding null is kept exactly as sent.
 * @param {Record<string, unknown>} record - Source record, treated as untrusted input.
 * @param {string[]} fields - Field names to keep, in the order they should be written.
 * @returns {Record<string, unknown>} A new object sharing nested values with the source, unmutated.
 */
export function project(record, fields) {
  const output = {};
  for (const field of fields) {
    if (Object.hasOwn(record, field)) output[field] = record[field];
  }
  return output;
}

/**
 * Orders two set or collector codes: numeric run first, lettered codes after and alphabetically
 * among themselves. Such codes are not integers — a numbered run is followed by lettered promos and
 * some records carry none at all — so a plain numeric sort mixes them or yields NaN. Text compares
 * by code unit, never `localeCompare`, whose collation follows the machine's ICU data and would make
 * output depend on who ran the script.
 * @param {unknown} a - First code: a number, a string like `12a` or `P1`, or null.
 * @param {unknown} b - Second code, same domain as `a`.
 * @returns {number} Per the `Array#sort` comparator contract.
 */
export function compareCodes(a, b) {
  // Rank 0 for digit-leading codes is what holds the numbered run ahead of promos like `P1`.
  const key = (raw) => {
    const text = raw == null ? "" : String(raw);
    const match = /^(\d+)(.*)$/.exec(text);
    return match ? [0, Number(match[1]), match[2]] : [1, 0, text];
  };
  const [rankA, numA, textA] = key(a);
  const [rankB, numB, textB] = key(b);
  if (rankA !== rankB) return rankA - rankB;
  if (numA !== numB) return numA - numB;
  return textA < textB ? -1 : textA > textB ? 1 : 0;
}

/**
 * Orders two cards into the snapshot's canonical sequence: set, then collector number, then id. The
 * id tiebreak makes the order total — collector numbers repeat across sets and can be null — and
 * only a total order keeps reruns byte-identical.
 *
 * Every leg runs through `compareCodes`, the id included, because ids are not always numeric: a
 * source keyed by 24-hex object ids would reduce `Number(id) - Number(id)` to NaN, which
 * `Array#sort` reads as zero, dropping the tiebreak and the byte-stability that depends on it.
 * @param {Record<string, unknown>} a - First card.
 * @param {Record<string, unknown>} b - Second card, same shape as `a`.
 * @returns {number} Per the `Array#sort` comparator contract.
 */
export function compareCards(a, b) {
  return compareCodes(a.setCode, b.setCode)
    || compareCodes(a.number, b.number)
    || compareCodes(a.id, b.id);
}

/**
 * Serializes ordered entries as a JSON object, honouring the given key order. Built by hand because
 * integer-like keys — card ids — are reordered into ascending numeric order by the engine whatever
 * the insertion order, discarding the sort the reviewer reads the diff by. Output is minified: a
 * snapshot is reviewed as a diff of whole records, which pretty-printing inflates without clarifying.
 * @param {Array<[string, unknown]>} entries - Key/value pairs in final order; keys must be unique.
 * @returns {string} A JSON object literal with no insignificant whitespace.
 */
export function serializeOrderedMap(entries) {
  return `{${entries.map(([k, v]) => `${JSON.stringify(k)}:${JSON.stringify(v)}`).join(",")}}`;
}
