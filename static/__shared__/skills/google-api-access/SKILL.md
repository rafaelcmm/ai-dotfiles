---
name: google-api-access
description: >-
  Set up, verify and troubleshoot authenticated access to Google's marketing and
  search APIs — Google Ads, GA4 (Data and Admin), Search Console, Merchant
  Center, Business Profile. Use this skill whenever a task needs to read or write
  any of those APIs and the credential is not already proven working: "connect to
  Google Ads", "authorize GA4", "set up the Google Ads MCP", "why is my Google
  API call returning 403", "ACCESS_TOKEN_SCOPE_INSUFFICIENT", "gcloud
  application-default login", "set up Search Console API", or any first-time
  Google API integration. Load it BEFORE writing auth code — the correct setup is
  non-obvious and the common failures are all silent.
---

# Google API access

One Application Default Credentials (ADC) login can serve Google Ads, GA4,
Search Console, Merchant Center and Business Profile at once. Getting there has
four non-obvious requirements, and missing any of them produces a credential
that **authenticates successfully and fails every API call** with a
permissions-shaped error that does not name the cause.

## The four requirements

1. **Use a Desktop OAuth client you created in the Cloud Console.** gcloud's
   built-in client is not authorized to request the `adwords` scope, and the
   refusal is not reported as a scope problem.
2. **Request every scope in one login.** Scopes do not imply each other — see
   [references/scopes.md](references/scopes.md). In particular
   `analytics.readonly` (Data API) and `analytics.edit` (Admin API) are both
   needed and neither covers the other.
3. **Tick every box on the consent screen.** One checkbox per scope, not all
   pre-ticked. An unticked box is indistinguishable from never requesting it.
4. **Verify what was actually granted.** This is the only step that converts a
   silent misconfiguration into an immediate, legible failure.

## Do this

```bash
# One-time: create a Desktop OAuth client in the Cloud Console and download its
# JSON, then enable the APIs on the project (see references/scopes.md).

export GOOGLE_OAUTH_CLIENT_FILE=~/path/to/oauth-desktop-client.json

scripts/google-adc-login.sh --profile ads,analytics,search-console
scripts/google-adc-doctor.sh
```

`google-adc-login.sh` requests the base scopes plus one set per profile
(`ads`, `analytics`, `search-console`, `merchant`, `business-profile`, or
`all`), then asks Google which scopes the resulting credential actually carries
and **exits non-zero if any are missing**.

`google-adc-doctor.sh` is read-only and safe against production. It separates
the failure causes that all look alike: missing credential file, wrong
credential type, expired refresh token, missing scope, missing developer token,
API not enabled, no quota project. Fix its **topmost** failure first — later
checks depend on earlier ones.

Never author the equivalent `gcloud` one-liner by hand in a chat message or a
README. Multi-line continuations get dropped, gcloud silently saves a
default-scope credential, and the result is the single most expensive failure in
this area. Call the script.

## Why a script and not a documented command

The failure it prevents is not hypothetical and not rare. A dropped line
continuation makes the shell run `--scopes=...` as a separate command; gcloud
completes the login with its own default scopes; the credential works for
`gcloud` itself and fails every Ads, Analytics and Search Console call with
`ACCESS_TOKEN_SCOPE_INSUFFICIENT`. That error reads as "this account lacks
permission", so the investigation starts in the wrong place — on account access
rather than on the credential.

## Credential type

Use an `authorized_user` credential (what `gcloud auth application-default
login` produces). **Do not reach for a service account to escape a permissions
error**: on these specific APIs a service account needs grants a user credential
does not — domain-wide delegation or an explicit Ads user grant for Ads, and
being added to the property for GA4. It replaces one grant problem with a harder
one.

Code that consumes ADC should assert `creds.type === "authorized_user"` and fail
loudly, rather than letting the API report a confusing `PERMISSION_DENIED` much
later.

## Environment contract

Copy [assets/env.google.example](assets/env.google.example) into the project's
gitignored `.env`. Two entries carry traps worth restating:

- **`GOOGLE_ADS_LOGIN_CUSTOMER_ID`** — set only when the target account sits
  under a manager (MCC). Leave it **unset**, not empty: sending a blank
  `login-customer-id` header is an error in itself.
- **`GOOGLE_APPLICATION_CREDENTIALS`** — set it explicitly even though the
  well-known path exists. Tools reading ADC through a client library find the
  default path; tools reading the variable directly do not.

## MCP servers

Google publishes read-only MCP servers for Ads and Analytics. Copy
[assets/mcp.google.json](assets/mcp.google.json) to the project root as
`.mcp.json` — it holds `${VAR}` placeholders only, so it is safe to commit.

Three facts that are each worth a debugging session:

- **The servers are read-only.** Neither can create, edit or delete a campaign or
  any GA4 configuration. Writes need a separate authenticated transport — see
  the `google-api-clients` skill.
- **Claude Code does not read `.env`.** It expands `${VAR}` from its own process
  environment, and an empty expansion is not an error: the server starts and
  fails every call. Launch from a shell that already carries the values —
  `direnv` (`echo dotenv > .envrc && direnv allow`) or
  `set -a && source .env && set +a && claude`.
- **They cache the credential in memory at startup.** After re-authorizing,
  reconnect them (`/mcp`) or the old scope set keeps failing.

Verify with `claude mcp list`, then ask for one live read before trusting it.

## Token handling

ADC stores a refresh token. Exchange it for a ~1-hour access token at
`https://oauth2.googleapis.com/token` with
`grant_type=refresh_token`. Short-lived scripts should fetch once at start and
reuse it for the process lifetime — refreshing per request only adds latency.
Long-lived processes must refresh near expiry; see the `oauth-token-lifecycle`
skill for a rotating-token store with a distributed lock.

Credentials are read from the environment or the credential file, **never from
argv** — an argument is visible in shell history and in any process listing.
That includes the HTTP layer: `curl -H "Authorization: Bearer $TOKEN"` and
`curl "...?access_token=$TOKEN"` both put a live credential in the process
table, where any other user on the host can read it. The scripts here feed curl
a config on stdin instead (`printf 'header = "..."' | curl -K -`), which never
reaches argv.

## When a call fails

Go to [references/troubleshooting.md](references/troubleshooting.md). It maps
each exact error string to its real cause. The ones that cost the most time:

- `ACCESS_TOKEN_SCOPE_INSUFFICIENT` → the login, not the account.
- An HTML `404` from Google Ads → the API version was sunset. Versions die on a
  rolling ~1-year cadence and a dead one is served by Google's generic front end
  rather than the API, so the body is HTML. Translate it into "set
  `GOOGLE_ADS_API_VERSION`" instead of printing the HTML.
- `SERVICE_DISABLED` on Search Console or Merchant → no quota project attached,
  or the API is not enabled on the project.
- Re-authorized and nothing changed → a long-lived process is holding the old
  credential in memory.

## Related skills

- `google-api-clients` — minimal REST transports for each of these APIs.
- `google-ads-operations` — safe mutation architecture once access works.
- `oauth-token-lifecycle` — token storage for long-lived server processes.
