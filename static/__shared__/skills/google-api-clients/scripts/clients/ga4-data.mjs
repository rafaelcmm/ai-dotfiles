/**
 * @fileoverview Minimal authenticated transport for the GA4 Data API (reporting).
 *
 * Shaped deliberately like `google-ads.mjs` and sharing its credential, because
 * Ads and GA4 only mean something together — Ads reports what was bought, GA4
 * reports what arrived. Reading both through one authorization in one process is
 * what makes a click-to-session reconciliation possible at all.
 *
 * Needs the `analytics.readonly` scope. Configuration (key events, custom
 * dimensions) lives in the Admin API and needs `analytics.edit` — see
 * `ga4-admin.mjs`.
 *
 * @module ga4-data
 */

import { authHeaders } from "./google-auth.mjs";

/** Root of the GA4 Data API. */
const API_ROOT = "https://analyticsdata.googleapis.com/v1beta";

/**
 * Creates a client bound to one GA4 property.
 *
 * @param {string|number} propertyId Numeric GA4 property id — **not** the
 *   `G-XXXXXXX` measurement id, which the Data API does not accept and reports
 *   only as a generic not-found.
 * @returns {Promise<object>} Client exposing `runReport` and `runRealtimeReport`.
 */
export async function createGa4DataClient(propertyId) {
  if (!/^\d+$/.test(String(propertyId))) {
    throw new Error(
      `propertyId must be the numeric GA4 property id, got "${propertyId}". ` +
        `A G-XXXXXXX measurement id is not accepted here.`
    );
  }

  const headers = await authHeaders();

  /**
   * Zips the API's positional value arrays back onto their names.
   *
   * `dimensionValues` and `metricValues` are positional and only meaningful
   * against the request that produced them. Every caller wants the named shape,
   * and doing it once here removes an easy class of off-by-one misreads where a
   * metric is charted under a neighbouring metric's label.
   *
   * @param {object} json Raw report response.
   * @param {string[]} dimensions Requested dimension names, in order.
   * @param {string[]} metrics Requested metric names, in order.
   * @returns {object[]} Rows keyed by name; metric values coerced to numbers.
   */
  function flattenRows(json, dimensions, metrics) {
    return (json.rows ?? []).map((row) => {
      const out = {};
      dimensions.forEach((name, i) => {
        out[name] = row.dimensionValues?.[i]?.value ?? "";
      });
      metrics.forEach((name, i) => {
        out[name] = Number(row.metricValues?.[i]?.value ?? 0);
      });
      return out;
    });
  }

  /**
   * Posts to a property-scoped endpoint.
   *
   * @param {string} method Endpoint suffix, e.g. `"runReport"`.
   * @param {object} body Request payload.
   * @returns {Promise<object>} Parsed response.
   * @throws {Error} Carrying the API's own message, which names the offending
   *   dimension or metric — the only thing that resolves an invalid combination.
   */
  async function post(method, body) {
    const res = await fetch(`${API_ROOT}/properties/${propertyId}:${method}`, {
      method: "POST",
      headers,
      body: JSON.stringify(body),
    });
    const text = await res.text();
    if (!res.ok) throw new Error(`GA4 ${method} failed (${res.status}):\n${text}`);
    return JSON.parse(text);
  }

  return {
    propertyId: String(propertyId),

    /**
     * Runs a report and returns rows flattened to plain objects.
     *
     * @param {{startDate: string, endDate: string}[]} dateRanges Inclusive ranges.
     * @param {string[]} dimensions Dimension names.
     * @param {string[]} metrics Metric names.
     * @param {{limit?: number, offset?: number, orderBys?: object[], dimensionFilter?: object, metricFilter?: object, keepEmptyRows?: boolean}} [options]
     * @returns {Promise<object[]>} Named rows.
     */
    async runReport(dateRanges, dimensions, metrics, options = {}) {
      const json = await post("runReport", {
        dateRanges,
        dimensions: dimensions.map((name) => ({ name })),
        metrics: metrics.map((name) => ({ name })),
        ...options,
      });
      return flattenRows(json, dimensions, metrics);
    },

    /**
     * Runs a realtime report (last ~30 minutes).
     *
     * Takes no date range by definition. Useful for verifying that a newly
     * deployed event or consent change is arriving, which is otherwise a 24-hour
     * wait on the standard reports.
     *
     * @param {string[]} dimensions Dimension names.
     * @param {string[]} metrics Metric names.
     * @param {{limit?: number, dimensionFilter?: object}} [options]
     * @returns {Promise<object[]>} Named rows.
     */
    async runRealtimeReport(dimensions, metrics, options = {}) {
      const json = await post("runRealtimeReport", {
        dimensions: dimensions.map((name) => ({ name })),
        metrics: metrics.map((name) => ({ name })),
        ...options,
      });
      return flattenRows(json, dimensions, metrics);
    },

    /**
     * Reports which dimensions and metrics this property actually accepts.
     *
     * Property-specific: custom dimensions registered in the admin UI appear
     * here and nowhere else. The usual reason a report rejects a parameter that
     * "exists" is that it was never registered — see the
     * `ga4-measurement-architecture` skill.
     *
     * @returns {Promise<{dimensions: string[], metrics: string[]}>}
     */
    async getMetadata() {
      const res = await fetch(`${API_ROOT}/properties/${propertyId}/metadata`, { headers });
      const text = await res.text();
      if (!res.ok) throw new Error(`GA4 metadata failed (${res.status}):\n${text}`);
      const json = JSON.parse(text);
      return {
        dimensions: (json.dimensions ?? []).map((d) => d.apiName),
        metrics: (json.metrics ?? []).map((m) => m.apiName),
      };
    },
  };
}
