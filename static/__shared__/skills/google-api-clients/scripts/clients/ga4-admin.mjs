/**
 * @fileoverview Minimal authenticated transport for the GA4 Admin API (configuration).
 *
 * Separate from `ga4-data.mjs` because the two need different scopes and the
 * split is not cosmetic: the Admin API **rejects** `analytics.readonly` and
 * requires `analytics.edit`. A setup authorized for reporting only will read
 * reports happily and fail every call here, so keeping them in one module would
 * invite the conclusion that the credential is broken rather than narrow.
 *
 * The operations exposed are the ones that are otherwise manual clicks in the
 * GA4 admin UI and are easy to forget entirely — in particular custom dimension
 * registration, without which an event parameter arrives, is stored, and appears
 * in **no** report, with no error anywhere.
 *
 * @module ga4-admin
 */

import { authHeaders } from "./google-auth.mjs";

/** Root of the GA4 Admin API. */
const API_ROOT = "https://analyticsadmin.googleapis.com/v1beta";

/**
 * Creates a client bound to one GA4 property.
 *
 * @param {string|number} propertyId Numeric GA4 property id.
 * @returns {Promise<object>} Client for key events and custom dimensions.
 */
export async function createGa4AdminClient(propertyId) {
  const headers = await authHeaders();
  const propertyPath = `properties/${propertyId}`;

  /**
   * Performs a request against the Admin API.
   *
   * @param {string} method HTTP method.
   * @param {string} path Path below the API root.
   * @param {object} [body] JSON payload for write methods.
   * @returns {Promise<object>} Parsed response, or `{}` on an empty body.
   * @throws {Error} With a scope hint on 403, because an insufficient-scope
   *   rejection here is near-certainly the readonly/edit confusion above rather
   *   than a genuine access problem.
   */
  async function request(method, path, body) {
    const res = await fetch(`${API_ROOT}/${path}`, {
      method,
      headers,
      ...(body ? { body: JSON.stringify(body) } : {}),
    });
    const text = await res.text();
    if (!res.ok) {
      const hint =
        res.status === 403
          ? "\nThe Admin API rejects analytics.readonly. Re-authorize with analytics.edit."
          : "";
      throw new Error(`GA4 Admin ${method} ${path} failed (${res.status}):\n${text}${hint}`);
    }
    return text ? JSON.parse(text) : {};
  }

  return {
    propertyId: String(propertyId),

    /**
     * Lists accounts and properties the credential can see.
     *
     * The cheapest proof that the analytics half of a credential works, and the
     * way to discover a numeric property id without the admin UI.
     *
     * @returns {Promise<object[]>} Account summaries.
     */
    async listAccountSummaries() {
      return (await request("GET", "accountSummaries?pageSize=200")).accountSummaries ?? [];
    },

    /**
     * Lists the property's data streams.
     *
     * @returns {Promise<object[]>} Data streams.
     */
    async listDataStreams() {
      return (await request("GET", `${propertyPath}/dataStreams?pageSize=200`)).dataStreams ?? [];
    },

    /**
     * Lists registered custom dimensions.
     *
     * @returns {Promise<object[]>} Custom dimensions.
     */
    async listCustomDimensions() {
      return (
        (await request("GET", `${propertyPath}/customDimensions?pageSize=200`)).customDimensions ??
        []
      );
    },

    /**
     * Registers a custom dimension, idempotently.
     *
     * Registration is required for an event parameter to be **reportable at
     * all**. Until it exists, the parameter is accepted, stored, and invisible
     * in every report — the single most common reason a correctly-emitted
     * custom parameter appears not to work.
     *
     * Existing dimensions are returned rather than re-created: GA4 caps custom
     * dimensions per property (50 event-scoped at the time of writing) and a
     * duplicate would consume a slot that cannot be reclaimed, since deleting a
     * custom dimension is irreversible and does not free its name for reuse.
     *
     * @param {{parameterName: string, displayName?: string, scope?: "EVENT"|"USER"|"ITEM", description?: string}} spec
     *   `scope` defaults to `EVENT`. **Item-scoped parameters must declare
     *   `ITEM`** — registering an item parameter as event-scoped produces a
     *   dimension that exists and is always empty.
     * @returns {Promise<{created: boolean, dimension: object}>}
     */
    async ensureCustomDimension({ parameterName, displayName, scope = "EVENT", description }) {
      if (!parameterName) throw new Error("parameterName is required");

      const existing = (await this.listCustomDimensions()).find(
        (d) => d.parameterName === parameterName && d.scope === scope
      );
      if (existing) return { created: false, dimension: existing };

      const dimension = await request("POST", `${propertyPath}/customDimensions`, {
        parameterName,
        displayName: displayName ?? parameterName,
        scope,
        ...(description ? { description } : {}),
      });
      return { created: true, dimension };
    },

    /**
     * Lists key events (what older docs call conversion events).
     *
     * @returns {Promise<object[]>} Key events.
     */
    async listKeyEvents() {
      return (await request("GET", `${propertyPath}/keyEvents?pageSize=200`)).keyEvents ?? [];
    },

    /**
     * Marks an event as a key event, idempotently.
     *
     * @param {string} eventName Event name exactly as emitted. GA4 does not
     *   validate that the event has ever been seen, so a typo creates a key
     *   event that can never fire and reports zero forever.
     * @returns {Promise<{created: boolean, keyEvent: object}>}
     */
    async ensureKeyEvent(eventName) {
      const existing = (await this.listKeyEvents()).find((e) => e.eventName === eventName);
      if (existing) return { created: false, keyEvent: existing };

      const keyEvent = await request("POST", `${propertyPath}/keyEvents`, {
        eventName,
        countingMethod: "ONCE_PER_EVENT",
      });
      return { created: true, keyEvent };
    },
  };
}
