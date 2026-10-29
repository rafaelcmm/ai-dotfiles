# Google API scope matrix

Scopes do not imply one another. A credential missing one scope fails only the
calls that need it, and the failure is reported as `ACCESS_TOKEN_SCOPE_INSUFFICIENT`
or a bare `403` — neither of which names the missing scope.

## Per-API requirements

| API | Scope | Notes |
| --- | --- | --- |
| Google Ads | `https://www.googleapis.com/auth/adwords` | Also needs a developer token header. **Cannot be requested through gcloud's built-in OAuth client** — a Desktop client is mandatory. |
| GA4 Data API (reporting) | `https://www.googleapis.com/auth/analytics.readonly` | Enough for every `runReport` / `runRealtimeReport`. |
| GA4 Admin API (config) | `https://www.googleapis.com/auth/analytics.edit` | **Rejects `analytics.readonly`.** Needed for key events, custom dimensions, data streams. |
| Search Console — read | `https://www.googleapis.com/auth/webmasters.readonly` | Search Analytics queries, site list. |
| Search Console — write | `https://www.googleapis.com/auth/webmasters` | Sitemap submission, URL inspection. Supersedes the readonly scope. |
| Merchant Center / Content API | `https://www.googleapis.com/auth/content` | One scope covers both the legacy Content API for Shopping and the newer Merchant API. |
| Business Profile | `https://www.googleapis.com/auth/business.manage` | Locations, reviews, posts. |

## Always include

gcloud grants these by default. Keep them in every login so a narrower
credential does not break unrelated gcloud usage that already relies on them.

```
openid
https://www.googleapis.com/auth/userinfo.email
https://www.googleapis.com/auth/cloud-platform
```

## The analytics pair

`analytics.readonly` and `analytics.edit` are both required for a complete
analytics setup and **neither implies the other**:

- Reporting (Data API) accepts `readonly`.
- Configuration (Admin API) rejects `readonly` and needs `edit`.

Requesting only one leaves half the tooling broken, and the half that breaks is
the half you reach for second — so the failure arrives long after the login that
caused it.

## Enabling the APIs

A granted scope is necessary but not sufficient: the API must also be enabled on
the Cloud project named by `GOOGLE_PROJECT_ID`. A granted scope against a
disabled API returns `SERVICE_DISABLED` / `403`, which reads like a scope
problem.

```bash
gcloud services enable \
  googleads.googleapis.com \
  analyticsdata.googleapis.com \
  analyticsadmin.googleapis.com \
  searchconsole.googleapis.com \
  shoppingcontent.googleapis.com \
  merchantapi.googleapis.com
```

## Quota project

Search Console and Merchant bill usage to a project and fail without one
attached to the credential. Google Ads does not need it.

```bash
gcloud auth application-default set-quota-project PROJECT_ID
```
