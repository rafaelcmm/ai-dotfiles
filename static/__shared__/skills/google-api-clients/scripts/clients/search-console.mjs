/**
 * @fileoverview Minimal authenticated transport for the Google Search Console API.
 *
 * Exists because Search Console work is otherwise done by hand in the UI, which
 * makes an SEO baseline unrepeatable: the numbers get screenshotted, the query
 * that produced them is lost, and the next audit cannot be compared to the last
 * one. Every method here returns plain data so a baseline can be committed.
 *
 * Needs the `webmasters` scope for sitemap submission and URL inspection, or
 * `webmasters.readonly` for queries alone. Also needs a quota project attached
 * to the credential — without one the API returns SERVICE_DISABLED, which reads
 * like a scope problem.
 *
 * @module search-console
 */

import { authHeaders } from "./google-auth.mjs";

/** Search Analytics and sitemaps live on the older webmasters surface. */
const WEBMASTERS_ROOT = "https://searchconsole.googleapis.com/webmasters/v3";

/** URL Inspection lives on the newer surface. */
const INSPECTION_ROOT = "https://searchconsole.googleapis.com/v1";

/**
 * Search Console caps a single Search Analytics response at 25,000 rows.
 *
 * Paging is therefore mandatory for anything catalogue-sized. A caller that
 * requests one page and stops gets a silent prefix of its own data, which is
 * worse than an error because the truncated numbers still look plausible.
 */
const MAX_ROWS_PER_PAGE = 25_000;

/**
 * Creates a client bound to one Search Console property.
 *
 * @param {string} siteUrl Property exactly as Search Console identifies it: a
 *   domain property is `sc-domain:example.com`; a URL-prefix property is the
 *   full origin **with** its trailing slash. These are different properties
 *   with different data, and passing the wrong form returns a 403 rather than
 *   an explanation.
 * @returns {Promise<object>} Client exposing query, sitemap and inspection
 *   methods.
 */
export async function createSearchConsoleClient(siteUrl) {
  if (!siteUrl) throw new Error("siteUrl is required, e.g. sc-domain:example.com");

  const headers = await authHeaders();
  const site = encodeURIComponent(siteUrl);

  /**
   * Performs a request and attaches a cause hint to the common failures.
   *
   * @param {string} method HTTP method.
   * @param {string} url Absolute URL.
   * @param {object} [body] JSON payload.
   * @returns {Promise<object>} Parsed response, or `{}` on an empty body.
   */
  async function request(method, url, body) {
    const res = await fetch(url, {
      method,
      headers,
      ...(body ? { body: JSON.stringify(body) } : {}),
    });
    const text = await res.text();
    if (!res.ok) {
      let hint = "";
      if (res.status === 403) {
        hint =
          `\nCheck: (1) ${siteUrl} is spelled exactly as the property is registered ` +
          `(sc-domain: vs URL-prefix with trailing slash), (2) the signed-in user has ` +
          `access to it, (3) a quota project is attached to the credential.`;
      }
      throw new Error(`Search Console ${method} failed (${res.status}):\n${text}${hint}`);
    }
    return text ? JSON.parse(text) : {};
  }

  return {
    siteUrl,

    /**
     * Lists the properties the credential can read.
     *
     * The authoritative way to get the exact property string, which is the
     * single most common source of 403s here.
     *
     * @returns {Promise<object[]>} Site entries with permission levels.
     */
    async listSites() {
      return (await request("GET", `${WEBMASTERS_ROOT}/sites`)).siteEntry ?? [];
    },

    /**
     * Runs a Search Analytics query, following every page.
     *
     * @param {{startDate: string, endDate: string, dimensions?: string[], type?: string, dimensionFilterGroups?: object[], rowLimit?: number, dataState?: string}} params
     *   `startDate`/`endDate` are `YYYY-MM-DD` and inclusive. Search Console
     *   data lags by ~2–3 days, so an `endDate` of today returns a partial or
     *   empty tail; pass `dataState: "all"` only when deliberately accepting
     *   fresh-but-incomplete rows.
     * @returns {Promise<object[]>} Rows with `keys` plus clicks, impressions,
     *   ctr and position.
     */
    async query(params) {
      const dimensions = params.dimensions ?? [];
      const pageSize = Math.min(params.rowLimit ?? MAX_ROWS_PER_PAGE, MAX_ROWS_PER_PAGE);
      const rows = [];

      // Paging stops on a short page rather than on a row count, because the
      // API gives no total and a full page is the only signal that more exist.
      for (let startRow = 0; ; startRow += pageSize) {
        const page = await request(
          "POST",
          `${WEBMASTERS_ROOT}/sites/${site}/searchAnalytics/query`,
          { ...params, dimensions, rowLimit: pageSize, startRow }
        );
        const pageRows = page.rows ?? [];
        rows.push(...pageRows);
        if (pageRows.length < pageSize) break;
        if (params.rowLimit && rows.length >= params.rowLimit) break;
      }

      return params.rowLimit ? rows.slice(0, params.rowLimit) : rows;
    },

    /**
     * Lists submitted sitemaps and their processing state.
     *
     * `warnings` and `errors` here are the only place a sitemap Google refuses
     * to fully process is reported — a submitted sitemap is not a processed one.
     *
     * @returns {Promise<object[]>} Sitemap entries.
     */
    async listSitemaps() {
      return (await request("GET", `${WEBMASTERS_ROOT}/sites/${site}/sitemaps`)).sitemap ?? [];
    },

    /**
     * Submits (or re-submits) a sitemap.
     *
     * Idempotent: re-submitting an existing sitemap asks Google to re-read it,
     * which is the intended action after a content change and not an error.
     *
     * @param {string} feedpath Absolute sitemap URL.
     * @returns {Promise<void>}
     */
    async submitSitemap(feedpath) {
      await request("PUT", `${WEBMASTERS_ROOT}/sites/${site}/sitemaps/${encodeURIComponent(feedpath)}`);
    },

    /**
     * Inspects one URL as Google sees it.
     *
     * This is the only programmatic source of truth for "is this indexed, and
     * which canonical did Google pick" — a self-canonical in the page's own
     * HTML proves nothing about which URL Google actually chose. Compare
     * `googleCanonical` against `userCanonical` to catch canonical-inheritance
     * bugs, which otherwise quietly remove pages from the index.
     *
     * Quota is tight (~2,000/day, ~600/minute per property), so inspect a
     * representative sample per template, not the whole catalogue.
     *
     * @param {string} inspectionUrl Absolute URL to inspect.
     * @param {string} [languageCode] BCP-47 language for the response text.
     * @returns {Promise<object>} The `inspectionResult` payload.
     */
    async inspectUrl(inspectionUrl, languageCode = "en-US") {
      const json = await request("POST", `${INSPECTION_ROOT}/urlInspection/index:inspect`, {
        inspectionUrl,
        siteUrl,
        languageCode,
      });
      return json.inspectionResult ?? json;
    },
  };
}
