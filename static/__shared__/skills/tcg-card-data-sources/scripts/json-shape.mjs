/**
 * @fileoverview Renders a parsed JSON value's shape as printable report lines: a one-token type
 * name, an aligned key listing, a top-level shape summary, and a one-line escaped text preview.
 *
 * Pure presentation over already-parsed data. It performs no I/O, holds no state and never exits,
 * so it can be exercised directly against a literal value. `probe-json-source.mjs` is its consumer.
 *
 * Zero dependencies. Node 20+.
 */

/** Top-level keys listed before the listing is elided, keeping a wide blob readable. */
const MAX_KEYS_SHOWN = 40;

/**
 * Names a value's JSON shape in one token, sized where size is the useful fact.
 * @param {unknown} value - Any parsed JSON value.
 * @returns {string} `null`, `array[12]`, `object`, or the primitive typeof name.
 */
export function typeName(value) {
  if (value === null) return "null";
  if (Array.isArray(value)) return `array[${value.length}]`;
  return typeof value;
}

/**
 * Renders an object's keys as aligned `key: type` lines, capped and accounted for.
 * @param {Record<string, unknown>} obj - Object whose own enumerable keys are listed.
 * @returns {string[]} Indented lines, ending with an elision count past {@link MAX_KEYS_SHOWN} so
 *   a truncated listing is never mistaken for the whole shape.
 */
export function keyLines(obj) {
  const keys = Object.keys(obj);
  const shown = keys.slice(0, MAX_KEYS_SHOWN);
  const width = Math.max(0, ...shown.map((k) => k.length));
  const lines = shown.map((k) => `    ${k.padEnd(width)}  ${typeName(obj[k])}`);
  if (keys.length > shown.length) {
    lines.push(`    ... ${keys.length - shown.length} more keys elided`);
  }
  return lines;
}

/**
 * Describes the top-level shape of a parsed payload.
 * @param {unknown} value - Parsed JSON root.
 * @returns {string[]} A summary line followed by key or element detail. For an array the first
 *   element's key set stands in for the record shape — the assumption every consumer makes.
 */
export function describeShape(value) {
  if (Array.isArray(value)) {
    const lines = [`array, ${value.length} elements`];
    const first = value[0];
    if (first && typeof first === "object" && !Array.isArray(first)) {
      lines.push("  keys of first element:", ...keyLines(first));
    } else if (value.length > 0) {
      lines.push(`  first element: ${typeName(first)}`);
    }
    return lines;
  }
  if (value && typeof value === "object") {
    return [`object, ${Object.keys(value).length} top-level keys`, ...keyLines(value)];
  }
  return [`${typeName(value)} - a bare scalar, not a record container`];
}

/**
 * Collapses text to a single escaped line for safe terminal display.
 * @param {string} text - Arbitrary body text, possibly HTML or binary-ish.
 * @param {number} limit - Characters retained before the ellipsis.
 * @returns {string} One line, every control character replaced by a printable escape, so an error
 *   page can neither scroll the report away nor emit terminal control sequences.
 */
export function escapeOneLine(text, limit) {
  // JSON string encoding is exactly the needed transform: every control character becomes
  // a printable escape and no newline can survive. Only its wrapping quotes are unwanted.
  const encoded = JSON.stringify(text.slice(0, limit)).slice(1, -1);
  return text.length > limit ? `${encoded}...` : encoded;
}
