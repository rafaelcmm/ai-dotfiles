/**
 * @fileoverview Minimal authenticated transport for Merchant Center.
 *
 * Exists because the Merchant Center UI is where feed problems are *reported*
 * but not where they can be *diffed*. The question that matters after a feed
 * change — which items were disapproved, and for what reason — is a scrollable
 * list in the UI and a comparable dataset here.
 *
 * Covers both surfaces deliberately. The Content API for Shopping (v2.1) is
 * long-lived and still carries the product-status and promotion endpoints most
 * accounts rely on; the Merchant API is its replacement and is where new
 * capability lands. Both authenticate with the same `content` scope, so the
 * split costs nothing and avoids pinning this file to whichever surface the
 * account happens to be migrated onto.
 *
 * @module merchant
 */

import { authHeaders } from "./google-auth.mjs";

/** Content API for Shopping. */
const CONTENT_ROOT = "https://shoppingcontent.googleapis.com/content/v2.1";

/** Merchant API (the successor surface). */
const MERCHANT_ROOT = "https://merchantapi.googleapis.com";

/**
 * Creates a client bound to one Merchant Center account.
 *
 * @param {string|number} merchantId Merchant Center account id. Under an
 *   advanced (multi-client) account this must be the **sub-account** id, not
 *   the parent's — the parent returns its own, usually empty, product list
 *   rather than an error.
 * @returns {Promise<object>} Client exposing status, issue and promotion reads.
 */
export async function createMerchantClient(merchantId) {
  if (!merchantId) throw new Error("merchantId is required");

  const headers = await authHeaders();

  /**
   * Performs a request against either Merchant surface.
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
      const hint =
        res.status === 403
          ? "\nCheck the account id (use the sub-account id under an advanced account), " +
            "that the user has access, and that a quota project is attached to the credential."
          : "";
      throw new Error(`Merchant ${method} failed (${res.status}):\n${text}${hint}`);
    }
    return text ? JSON.parse(text) : {};
  }

  /**
   * Follows `nextPageToken` across every page of a list endpoint.
   *
   * Written as a generic helper because every Merchant list endpoint paginates
   * and the page size is small relative to a real catalogue — a single-page read
   * silently returns a prefix, which looks like a shrinking catalogue rather
   * than a truncated query.
   *
   * @param {string} baseUrl URL without a page token.
   * @param {string} collection Response property holding the array.
   * @returns {Promise<object[]>} Every item across all pages.
   */
  async function listAll(baseUrl, collection) {
    const items = [];
    let pageToken;
    do {
      const sep = baseUrl.includes("?") ? "&" : "?";
      const url = pageToken ? `${baseUrl}${sep}pageToken=${encodeURIComponent(pageToken)}` : baseUrl;
      const page = await request("GET", url);
      items.push(...(page[collection] ?? []));
      pageToken = page.nextPageToken;
    } while (pageToken);
    return items;
  }

  return {
    merchantId: String(merchantId),

    /**
     * Returns the account ids this credential can act on.
     *
     * No-argument read, so it is the cheapest proof that the `content` scope and
     * the quota project are both in place before any account id is guessed at.
     *
     * @returns {Promise<object>} Auth info including account identifiers.
     */
    async authInfo() {
      return request("GET", `${CONTENT_ROOT}/accounts/authinfo`);
    },

    /**
     * Lists the per-product status of every item in the account.
     *
     * This — not the product list — is what answers "is my feed actually
     * serving". A product can be present, valid and still not served: the
     * destination statuses and `itemLevelIssues` are where that is reported.
     *
     * @returns {Promise<object[]>} Product status records.
     */
    async listProductStatuses() {
      return listAll(
        `${CONTENT_ROOT}/${this.merchantId}/productstatuses?maxResults=250`,
        "resources"
      );
    },

    /**
     * Summarises item-level issues by code.
     *
     * Raw statuses are one record per product, which for a real catalogue is
     * thousands of rows saying a handful of distinct things. Grouping by issue
     * code is what turns it into a work list, and keeping a sample handle per
     * code is what makes each one reproducible.
     *
     * @param {object[]} [statuses] Pre-fetched statuses; fetched when omitted.
     * @returns {Promise<{code: string, description: string, servability: string, count: number, examples: string[]}[]>}
     *   Issues ordered by affected-product count, descending.
     */
    async summarizeItemIssues(statuses) {
      const records = statuses ?? (await this.listProductStatuses());
      const byCode = new Map();

      for (const record of records) {
        for (const issue of record.itemLevelIssues ?? []) {
          const existing = byCode.get(issue.code) ?? {
            code: issue.code,
            description: issue.description ?? "",
            servability: issue.servability ?? "",
            count: 0,
            examples: [],
          };
          existing.count += 1;
          // Three examples is enough to spot a pattern and few enough to read.
          if (existing.examples.length < 3) existing.examples.push(record.productId ?? "");
          byCode.set(issue.code, existing);
        }
      }

      return [...byCode.values()].sort((a, b) => b.count - a.count);
    },

    /**
     * Lists account-level issues.
     *
     * Account-level problems (policy, verification, claim) suppress serving
     * regardless of how clean the product data is, so they must be read before
     * any item-level investigation — otherwise the conclusion is "the feed is
     * broken" when the feed is fine and the account is suspended.
     *
     * @returns {Promise<object[]>} Account status issues.
     */
    async listAccountIssues() {
      const json = await request("GET", `${CONTENT_ROOT}/${this.merchantId}/accountstatuses/${this.merchantId}`);
      return json.accountLevelIssues ?? [];
    },

    /**
     * Lists the account's data sources (feeds) via the Merchant API.
     *
     * Reports each source's fetch schedule and last-fetch outcome, which is
     * where a silently stale feed shows up — a feed Google stopped fetching
     * keeps serving its last good version until it expires, so the catalogue
     * looks healthy right up to the moment it empties.
     *
     * @returns {Promise<object[]>} Data sources.
     */
    async listDataSources() {
      return listAll(
        `${MERCHANT_ROOT}/datasources/v1beta/accounts/${this.merchantId}/dataSources`,
        "dataSources"
      );
    },

    /**
     * Lists promotions registered on the account.
     *
     * Shopping ads have no headline — the ad is image, title, price and store
     * name, all from the feed. A promotion is the only way a discount is visible
     * in the ad itself, so its absence during a sale is a silent loss rather
     * than an error.
     *
     * @returns {Promise<object[]>} Promotions.
     */
    async listPromotions() {
      return listAll(
        `${MERCHANT_ROOT}/promotions/v1beta/accounts/${this.merchantId}/promotions`,
        "promotions"
      );
    },
  };
}
