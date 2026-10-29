/**
 * @fileoverview Application Default Credentials → bearer access token.
 *
 * The one place in this skill that reads credentials, so every client below it
 * stays a pure HTTP shape. Shared rather than duplicated per client because the
 * useful property is that Ads, GA4, Search Console and Merchant all authenticate
 * through the *same* credential: a report that correlates spend with sessions
 * only means something if both halves came from one authorization.
 *
 * Only the `authorized_user` credential type is accepted. A service account is
 * rejected up front with the reason, because on these specific APIs it needs
 * grants a user credential does not (domain-wide delegation or an explicit Ads
 * user grant; being added to the GA4 property) and the API's own report of that
 * is an undifferentiated PERMISSION_DENIED much later in the call stack.
 *
 * Credentials are read from the credentials file only — never from argv, which
 * would expose them in shell history and in any process listing.
 *
 * Also runnable as a CLI, which is the supported way to get a token for an
 * ad-hoc `curl`:
 *   node google-auth.mjs
 *
 * @module google-auth
 */

import { readFile } from "node:fs/promises";
import { homedir } from "node:os";
import { join } from "node:path";

/** Google's OAuth token endpoint. */
const TOKEN_URL = "https://oauth2.googleapis.com/token";

/**
 * Well-known ADC location, used when `GOOGLE_APPLICATION_CREDENTIALS` is unset.
 *
 * Checked as a fallback rather than being required, because client libraries
 * find this path without the variable and plenty of working setups never set
 * it — refusing to start in that case would be a false negative.
 */
const DEFAULT_ADC_PATH = join(homedir(), ".config", "gcloud", "application_default_credentials.json");

/**
 * Process-lifetime token cache.
 *
 * Access tokens live roughly an hour. A script finishing in seconds would gain
 * nothing from a second exchange and would pay a round trip for it, so the first
 * token is reused. `expiresAt` exists for the longer-running case: a process
 * that outlives the token refreshes instead of failing at the 60-minute mark.
 */
let cached = null;

/** Refresh this far before nominal expiry, so a token never expires in flight. */
const EXPIRY_MARGIN_MS = 60_000;

/**
 * Loads and validates the ADC credential file.
 *
 * @returns {Promise<{client_id: string, client_secret: string, refresh_token: string}>}
 * @throws {Error} When the file is absent, unreadable, not JSON, or not an
 *   `authorized_user` credential — each with the command that fixes it, since a
 *   generic failure here sends the reader to the API's permissions model instead
 *   of to their own login.
 */
async function loadCredentials() {
  const path = process.env.GOOGLE_APPLICATION_CREDENTIALS || DEFAULT_ADC_PATH;

  let raw;
  try {
    raw = await readFile(path, "utf8");
  } catch (err) {
    throw new Error(
      `Could not read Google credentials at ${path}: ${err.message}\n` +
        `Run: gcloud auth application-default login (see the google-api-access skill)`
    );
  }

  let creds;
  try {
    creds = JSON.parse(raw);
  } catch {
    throw new Error(`Google credentials at ${path} are not valid JSON`);
  }

  if (creds.type !== "authorized_user") {
    throw new Error(
      `Unsupported credential type "${creds.type}" at ${path}.\n` +
        `Google Ads needs domain-wide delegation or an explicit Ads user grant for a ` +
        `service account, and GA4 needs it added to the property. Use a user credential:\n` +
        `  gcloud auth application-default login`
    );
  }

  for (const field of ["client_id", "client_secret", "refresh_token"]) {
    if (!creds[field]) throw new Error(`Google credentials at ${path} are missing ${field}`);
  }

  return creds;
}

/**
 * Returns a bearer access token for the ADC credential.
 *
 * @param {{forceRefresh?: boolean}} [options] `forceRefresh` bypasses the cache;
 *   used by tests and by a caller that has just observed a 401.
 * @returns {Promise<string>} A bearer token valid for roughly one hour.
 * @throws {Error} When credentials are unusable or the refresh exchange is
 *   rejected. An `invalid_grant` is surfaced with its meaning attached: the
 *   refresh token was revoked or expired, which is a re-login, not a retry.
 */
export async function getAccessToken({ forceRefresh = false } = {}) {
  if (!forceRefresh && cached && cached.expiresAt - EXPIRY_MARGIN_MS > Date.now()) {
    return cached.token;
  }

  const creds = await loadCredentials();

  const res = await fetch(TOKEN_URL, {
    method: "POST",
    headers: { "Content-Type": "application/x-www-form-urlencoded" },
    body: new URLSearchParams({
      client_id: creds.client_id,
      client_secret: creds.client_secret,
      refresh_token: creds.refresh_token,
      grant_type: "refresh_token",
    }),
  });

  const text = await res.text();
  if (!res.ok) {
    // The response body can echo request parameters, so it is not forwarded.
    // Only the error code is, which is the part that distinguishes the causes.
    let code = "";
    try {
      code = JSON.parse(text).error ?? "";
    } catch {
      /* non-JSON error body; the status is the only signal */
    }
    const hint =
      code === "invalid_grant"
        ? "The refresh token was revoked or expired (password change, 6 months unused, or an " +
          "OAuth consent screen still in Testing mode, which expires refresh tokens after 7 days)."
        : "Re-run: gcloud auth application-default login";
    throw new Error(`Google token refresh failed (${res.status} ${code || "unknown"}). ${hint}`);
  }

  const json = JSON.parse(text);
  cached = {
    token: json.access_token,
    expiresAt: Date.now() + Number(json.expires_in ?? 3600) * 1000,
  };
  return cached.token;
}

/**
 * Builds the standard authorization headers for a JSON Google API call.
 *
 * @param {Record<string, string>} [extra] Additional headers, e.g. Ads'
 *   `developer-token`.
 * @returns {Promise<Record<string, string>>} Headers ready to pass to `fetch`.
 */
export async function authHeaders(extra = {}) {
  return {
    Authorization: `Bearer ${await getAccessToken()}`,
    "Content-Type": "application/json",
    ...extra,
  };
}

/**
 * Test seam: drops the cached token so the next call re-exchanges.
 *
 * Module-level state that legitimately lives for the process lifetime is
 * otherwise untestable in isolation.
 */
export function __resetTokenCacheForTests() {
  cached = null;
}

// CLI: print a token. Guarded on being the entry module so importing this file
// never performs I/O as a side effect.
if (import.meta.url === `file://${process.argv[1]}`) {
  getAccessToken()
    .then((token) => {
      process.stdout.write(`${token}\n`);
    })
    .catch((err) => {
      process.stderr.write(`${err.message}\n`);
      process.exit(1);
    });
}
