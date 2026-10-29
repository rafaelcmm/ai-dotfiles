# Google API auth troubleshooting

Every misconfiguration in this area surfaces as a permissions-shaped error, so
the symptom never names the cause. Match on the exact string.

| Symptom | Actual cause | Fix |
| --- | --- | --- |
| `ACCESS_TOKEN_SCOPE_INSUFFICIENT` | The credential does not carry the scope that call needs. Most often because a multi-line `gcloud ... --scopes=` was pasted into a shell that dropped the continuations, so gcloud saved a **default-scope** credential that authenticates fine. | Re-run `scripts/google-adc-login.sh` with the right profile. It verifies the granted scopes and exits non-zero if any are missing. |
| Consent screen refuses the `adwords` scope | The login used gcloud's built-in OAuth client. It is not authorized for the Ads scope, and the rejection is not reported as a scope problem. | Create a **Desktop** OAuth client in the Cloud Console and pass `--client-id-file`. |
| Scopes look requested but tokeninfo shows fewer | The consent screen renders one checkbox per scope, and they are not all pre-ticked. An unticked box is indistinguishable from never requesting it. | Re-run the login and tick every box. |
| `PERMISSION_DENIED` on Ads or GA4 with a valid-looking credential | The credential is a **service account**. Ads additionally needs domain-wide delegation or an explicit Ads user grant; GA4 needs the service account added to the property. | Use an `authorized_user` credential, or complete the extra grant. Prefer the former. |
| HTML `404` with `<!DOCTYPE html>` from Google Ads | The pinned API version has been **sunset**. Google sunsets on a rolling ~1-year cadence and a dead version is served by the generic front end, not the API. | Set `GOOGLE_ADS_API_VERSION` to a live version and re-run. |
| `DEVELOPER_TOKEN_NOT_APPROVED` / works on one account only | The developer token is at *test account* access level. | Apply for Basic access in the MCC's API Center. Until then only test accounts respond. |
| `login-customer-id` errors, or the call reads the wrong account | A **blank** `login-customer-id` header was sent. An empty value is an error, not an omission. | Omit the header entirely when there is no manager account. |
| `SERVICE_DISABLED` on Search Console or Merchant | No quota project attached, or the API is not enabled on the project. | `gcloud auth application-default set-quota-project PROJECT_ID` and `gcloud services enable ...`. |
| MCP server connects but every call fails | Claude Code does not read `.env`. `.mcp.json` expands `${VAR}` from Claude's own process environment, and an empty expansion is not an error — the server starts and fails each call. | Launch Claude from a shell that already carries the values: direnv, or `set -a && source .env && set +a && claude`. |
| Re-authorized, but the same scope error persists | The MCP server (or any long-lived process) cached the previous credential **in memory** at startup. | Reconnect the servers (`/mcp`) or restart the process. |
| `invalid_grant` on token refresh | Refresh token revoked or expired: password change, 6 months unused, or a testing-mode OAuth consent screen (7-day refresh token expiry). | Re-run the login. If it recurs weekly, publish the OAuth consent screen out of Testing. |
| Token works, then stops roughly hourly | An access token was cached past its ~1-hour life. | Refresh per run, or refresh when the cached token is near expiry. Short-lived scripts can fetch once at start. |

## Diagnostic order

Run `scripts/google-adc-doctor.sh` first. It separates the causes above with
read-only probes and no account ids. Fix the **topmost** failure it reports —
later checks depend on earlier ones and keep failing until it clears.

## What never to do

- Do not switch to a service account to escape a scope error. It replaces one
  grant problem with a harder one on these specific APIs.
- Do not pass credentials as command-line arguments. They land in shell history
  and in any process listing. Read them from the environment or a file.
- Do not hardcode an API version as "latest" by string. Pin it, and let the
  client translate the sunset 404 into an actionable message.
