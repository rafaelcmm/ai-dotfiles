/**
 * @fileoverview Minimal authenticated transport for the Google Ads REST API.
 *
 * Exists so operational scripts can read and mutate an account without pulling
 * in the official Google Ads SDK — a large gRPC dependency whose only use in a
 * handful of one-shot REST calls would outweigh the scripts it serves.
 *
 * Two behaviours here are load-bearing and should not be simplified away:
 *
 *   1. **Version pinning with sunset translation.** Google sunsets API versions
 *      on a rolling ~1-year cadence, and a sunset version does not degrade — it
 *      is served by Google's generic front end as an HTML 404. Dumping that HTML
 *      tells an operator nothing, so it is translated into the one action that
 *      fixes it.
 *   2. **A preflight read before any mutation.** One cheap query proves the API
 *      version exists, the developer token is accepted and the account is
 *      reachable. Without it, a bad version fails on the first write instead,
 *      which is only harmless when the first write is the first operation — a
 *      version rejected mid-sequence strands a half-built campaign.
 *
 * @module google-ads
 */

import { authHeaders } from "./google-auth.mjs";

/**
 * API version this client targets.
 *
 * Pinned, never "latest": sunset is the observed failure mode, so the newest
 * version verified against a real account is the safest default because it has
 * the most remaining life. Override with `GOOGLE_ADS_API_VERSION` when it
 * eventually sunsets; {@link createGoogleAdsClient} detects that case and
 * reports it as an actionable message.
 */
const API_VERSION = process.env.GOOGLE_ADS_API_VERSION ?? "v25";

/** Root of the Google Ads REST surface. */
const API_ROOT = `https://googleads.googleapis.com/${API_VERSION}`;

/** Currency amounts cross the API in millionths of a unit. */
export const MICROS = 1_000_000;

/**
 * Converts a currency amount to the micros the API expects.
 *
 * Rounded, not truncated: the API rejects a non-integer, and truncation would
 * silently under-bid by up to one micro on every call.
 *
 * @param {number} amount Amount in major currency units.
 * @returns {number} Integer micros.
 */
export const toMicros = (amount) => Math.round(amount * MICROS);

/**
 * Converts micros back to major currency units.
 *
 * @param {number|string|null|undefined} micros Raw API value; absent reads as 0
 *   because the API omits zero-valued metrics rather than returning them.
 * @returns {number} Amount in major currency units.
 */
export const fromMicros = (micros) => Number(micros ?? 0) / MICROS;

/**
 * Recognizes a sunset API version from a failed response.
 *
 * A dead version is answered by Google's generic web front end, so the tell is
 * an HTML body on a 404 rather than anything in the JSON error model.
 *
 * @param {number} status HTTP status.
 * @param {string} body Response body.
 * @returns {boolean} True when the version is most likely sunset.
 */
function isSunsetVersion(status, body) {
  return status === 404 && body.includes("<!DOCTYPE html>");
}

/** The message that actually resolves a sunset version. */
function sunsetMessage() {
  return (
    `Google Ads API ${API_VERSION} is not available — it has most likely been sunset.\n` +
    `Set GOOGLE_ADS_API_VERSION to a supported version and re-run.`
  );
}

/**
 * Creates a client bound to one Google Ads customer account.
 *
 * The account id is a required argument rather than an environment variable so
 * that a stale shell cannot silently steer a mutation at the wrong account.
 *
 * @param {string} customerId Target account id, digits only, no hyphens.
 * @param {{skipPreflight?: boolean}} [options] `skipPreflight` omits the
 *   verification read. Only for a strictly read-only session where the first
 *   call is itself harmless.
 * @returns {Promise<object>} Client exposing `search`, `searchStream`, `post`
 *   and `mutate`.
 * @throws {Error} When the developer token is absent, the version is sunset, or
 *   the account is unreachable.
 */
export async function createGoogleAdsClient(customerId, { skipPreflight = false } = {}) {
  const developerToken = process.env.GOOGLE_ADS_DEVELOPER_TOKEN;
  if (!developerToken) {
    throw new Error(
      "GOOGLE_ADS_DEVELOPER_TOKEN is not set. Get one from a Manager (MCC) account: " +
        "Tools → API Center. See the google-api-access skill."
    );
  }
  if (!/^\d{10}$/.test(String(customerId))) {
    throw new Error(`customerId must be 10 digits without hyphens, got "${customerId}"`);
  }

  const loginCustomerId = process.env.GOOGLE_ADS_LOGIN_CUSTOMER_ID;
  const headers = await authHeaders({
    "developer-token": developerToken,
    // Required when the target account sits under a manager, and omitted
    // entirely when absent — sending a blank value is itself an error, not an
    // omission, so a falsy env var must not become an empty header.
    ...(loginCustomerId ? { "login-customer-id": loginCustomerId } : {}),
  });

  /**
   * Issues a POST against a customer-scoped endpoint.
   *
   * Errors carry the API's own body rather than a bare status: Google Ads
   * returns field-level diagnostics (which criterion was malformed, which
   * resource conflicted) that are the difference between a fixable and an
   * inscrutable failure.
   *
   * @param {string} endpoint Path after the customer segment, e.g. `"campaigns:mutate"`.
   * @param {object} body JSON payload.
   * @returns {Promise<object>} Parsed response, or `{}` on an empty body.
   */
  async function post(endpoint, body) {
    const res = await fetch(`${API_ROOT}/customers/${customerId}/${endpoint}`, {
      method: "POST",
      headers,
      body: JSON.stringify(body),
    });

    const text = await res.text();
    if (!res.ok) {
      if (isSunsetVersion(res.status, text)) throw new Error(sunsetMessage());
      throw new Error(`${endpoint} failed (${res.status}):\n${text}`);
    }
    return text ? JSON.parse(text) : {};
  }

  if (!skipPreflight) {
    try {
      await post("googleAds:search", { query: "SELECT customer.id FROM customer LIMIT 1" });
    } catch (err) {
      throw new Error(`Cannot reach account ${customerId} on ${API_VERSION}.\n${err.message}`);
    }
  }

  return {
    customerId,
    apiVersion: API_VERSION,
    post,

    /**
     * Runs a GAQL query and returns the result rows.
     *
     * @param {string} query GAQL query string.
     * @returns {Promise<object[]>} Rows; empty array when nothing matched.
     */
    async search(query) {
      return (await post("googleAds:search", { query })).results ?? [];
    },

    /**
     * Runs a GAQL query through the streaming endpoint.
     *
     * The non-streaming endpoint paginates, so a report wider than one page
     * silently returns a prefix unless the caller follows `nextPageToken`.
     * Streaming returns an array of batches instead, which is flattened here —
     * use this for anything report-shaped and `search` only for bounded lookups.
     *
     * @param {string} query GAQL query string.
     * @returns {Promise<object[]>} All rows across every batch.
     */
    async searchStream(query) {
      const res = await fetch(`${API_ROOT}/customers/${customerId}/googleAds:searchStream`, {
        method: "POST",
        headers,
        body: JSON.stringify({ query }),
      });
      const text = await res.text();
      if (!res.ok) {
        if (isSunsetVersion(res.status, text)) throw new Error(sunsetMessage());
        throw new Error(`searchStream failed (${res.status}):\n${text}`);
      }
      const batches = JSON.parse(text);
      return (Array.isArray(batches) ? batches : [batches]).flatMap((b) => b.results ?? []);
    },

    /**
     * Issues a mutate, optionally as a server-side validation only.
     *
     * `validateOnly: true` is what makes a dry run meaningful: Google type-checks
     * every field and rejects a malformed operation **without writing**, so the
     * dry run exercises the real API rather than a local simulation of it.
     *
     * `partialFailure` is left off by default on purpose. A half-applied batch is
     * harder to reason about — and harder to resume — than an outright failure
     * with nothing written.
     *
     * @param {string} resource Resource segment, e.g. `"campaigns"`.
     * @param {object[]} operations Mutate operations.
     * @param {{validateOnly?: boolean, partialFailure?: boolean}} [options]
     * @returns {Promise<object>} The mutate response.
     */
    async mutate(resource, operations, { validateOnly = false, partialFailure = false } = {}) {
      return post(`${resource}:mutate`, { operations, validateOnly, partialFailure });
    },
  };
}
