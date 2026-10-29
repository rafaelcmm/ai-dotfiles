#!/usr/bin/env bash
#
# Diagnose an Application Default Credentials setup against the Google APIs that
# actually matter for ads, analytics, search and shopping work.
#
# Strictly read-only: it exchanges the refresh token, asks Google which scopes
# the credential carries, and makes one cheap no-argument GET per API. Nothing is
# written and no account is mutated, so it is safe to run against production.
#
# It exists because every distinct misconfiguration in this area produces the
# same user-visible symptom — a call that fails with a permissions-shaped error.
# The actual cause is one of: no credential file, a service-account credential
# where a user credential is required, an expired refresh token, a missing scope,
# a missing developer token, an API not enabled on the Cloud project, or a
# missing quota project. This separates them.
#
# Usage:
#   google-adc-doctor.sh
#   google-adc-doctor.sh --json      # machine-readable summary on stdout
#
# Exit codes: 0 all checks passed; 1 at least one check failed.
#
# `-e` is deliberately omitted. A diagnostic must run every check and report the
# full picture; aborting on the first failing probe would hide the rest, which is
# the opposite of the job. Failures are accumulated by `record` and surfaced
# through the exit code at the end instead.
set -uo pipefail

readonly TOKEN_URL="https://oauth2.googleapis.com/token"
readonly TOKENINFO_URL="https://oauth2.googleapis.com/tokeninfo"

json_mode=0
[[ "${1:-}" == "--json" ]] && json_mode=1

# Accumulates "name<TAB>status<TAB>detail" rows so the report is rendered once,
# at the end, in a stable order — a check that printed as it ran would interleave
# with curl's own diagnostics.
results=()
failed=0

record() {
  results+=("$1	$2	$3")
  [[ "$2" == "FAIL" ]] && failed=1
  return 0
}

have() { command -v "$1" >/dev/null 2>&1; }

# Reads one top-level string field out of a JSON document. python3 when
# available, otherwise a tolerant sed. Deliberately not jq: jq is the cleanest
# option but is absent often enough that depending on it would turn this
# diagnostic into another thing that needs diagnosing.
json_field() {
  local body="$1" field="$2"
  if have python3; then
    printf '%s' "$body" | python3 -c "
import json,sys
try:
    doc = json.load(sys.stdin)
except Exception:
    sys.exit(0)
value = doc.get('$field', '')
print(value if isinstance(value, str) else json.dumps(value))
" 2>/dev/null
  else
    printf '%s' "$body" | sed -n "s/.*\"$field\"[[:space:]]*:[[:space:]]*\"\([^\"]*\)\".*/\1/p"
  fi
}

# --- 1. gcloud -------------------------------------------------------------
if have gcloud; then
  record "gcloud CLI" "OK" "$(gcloud --version 2>/dev/null | head -1)"
else
  record "gcloud CLI" "WARN" "not installed - re-authorizing will not be possible from this host"
fi

# --- 2. credential file ----------------------------------------------------
creds_path="${GOOGLE_APPLICATION_CREDENTIALS:-$HOME/.config/gcloud/application_default_credentials.json}"
if [[ -f "$creds_path" ]]; then
  if [[ -n "${GOOGLE_APPLICATION_CREDENTIALS:-}" ]]; then
    record "credential file" "OK" "$creds_path (from GOOGLE_APPLICATION_CREDENTIALS)"
  else
    # Tools that read ADC through a library find the well-known path on their
    # own; tools that read the env var directly do not. Both patterns are
    # common, so an unset variable is reported rather than ignored.
    record "credential file" "WARN" "$creds_path (well-known path; GOOGLE_APPLICATION_CREDENTIALS is unset)"
  fi
else
  record "credential file" "FAIL" "not found at $creds_path - run google-adc-login.sh"
  creds_path=""
fi

creds_body=""
cred_type=""
if [[ -n "$creds_path" ]]; then
  creds_body="$(cat "$creds_path" 2>/dev/null)"
  cred_type="$(json_field "$creds_body" "type")"

  case "$cred_type" in
    authorized_user)
      record "credential type" "OK" "authorized_user"
      ;;
    service_account)
      # Not a generic warning: for these specific APIs a service account needs
      # extra grants that a user credential does not, and the resulting failure
      # is reported by the API as a plain permission denial.
      record "credential type" "FAIL" "service_account - Google Ads needs domain-wide delegation or an explicit Ads user grant, and GA4 needs the service account added to the property. Prefer authorized_user."
      ;;
    "")
      record "credential type" "FAIL" "could not read 'type' - file is not valid JSON"
      ;;
    *)
      record "credential type" "FAIL" "unsupported type '$cred_type'"
      ;;
  esac

  quota_project="$(json_field "$creds_body" "quota_project_id")"
  if [[ -n "$quota_project" ]]; then
    record "quota project" "OK" "$quota_project"
  else
    # Ads does not need it; Search Console and Merchant bill usage to a project
    # and return a confusing SERVICE_DISABLED / project-not-found without it.
    record "quota project" "WARN" "unset - Search Console and Merchant calls may fail with SERVICE_DISABLED. Fix: gcloud auth application-default set-quota-project PROJECT_ID"
  fi
fi

# --- 3. token exchange -----------------------------------------------------
access_token=""
if [[ "$cred_type" == "authorized_user" ]]; then
  client_id="$(json_field "$creds_body" "client_id")"
  client_secret="$(json_field "$creds_body" "client_secret")"
  refresh_token="$(json_field "$creds_body" "refresh_token")"

  if [[ -z "$client_id" || -z "$client_secret" || -z "$refresh_token" ]]; then
    record "token exchange" "FAIL" "credential is missing client_id, client_secret or refresh_token"
  else
    # Every credential is fed to curl through a stdin config file. `curl
    # --data-urlencode "client_secret=..."` would put the secret in argv, where
    # any other user on the host can read it out of `ps` for the life of the
    # process. A config on stdin never reaches the process table.
    token_body="$(
      {
        printf 'url = "%s"\n' "$TOKEN_URL"
        printf 'data-urlencode = "client_id=%s"\n' "$client_id"
        printf 'data-urlencode = "client_secret=%s"\n' "$client_secret"
        printf 'data-urlencode = "refresh_token=%s"\n' "$refresh_token"
        printf 'data-urlencode = "grant_type=refresh_token"\n'
      } | curl -sS -K - 2>/dev/null
    )"
    access_token="$(json_field "$token_body" "access_token")"
    if [[ -n "$access_token" ]]; then
      record "token exchange" "OK" "refresh token is still valid"
    else
      err="$(json_field "$token_body" "error")"
      record "token exchange" "FAIL" "refresh failed (${err:-unknown}) - credential expired or revoked; re-run google-adc-login.sh"
    fi
  fi
fi

# --- 4. granted scopes -----------------------------------------------------
granted=""
if [[ -n "$access_token" ]]; then
  tokeninfo_body="$(
    printf 'url = "%s?access_token=%s"\n' "$TOKENINFO_URL" "$access_token" | curl -sS -K - 2>/dev/null
  )"
  granted="$(json_field "$tokeninfo_body" "scope")"
  if [[ -n "$granted" ]]; then
    record "granted scopes" "OK" "$(printf '%s' "$granted" | tr ' ' '\n' | sed '/^$/d' | wc -l | tr -d ' ') scopes"
  else
    record "granted scopes" "FAIL" "tokeninfo returned no scope list"
  fi
fi

has_scope() {
  case " $granted " in
    *" $1 "*) return 0 ;;
    *) return 1 ;;
  esac
}

# --- 5. per-API reachability ----------------------------------------------
# Each probe is the cheapest no-argument read the API offers, so the check needs
# no account id and cannot be confused by one that is simply wrong.
# Extra headers are passed as bare "Name: value" strings rather than as curl
# arguments, because one of them carries the Ads developer token — a credential,
# and therefore something that must never appear in argv.
probe() {
  local name="$1" scope="$2" url="$3"
  shift 3

  if [[ -z "$access_token" ]]; then
    record "$name" "SKIP" "no access token"
    return
  fi
  if ! has_scope "$scope"; then
    record "$name" "FAIL" "scope not granted: $scope"
    return
  fi

  local status
  status="$(
    {
      printf 'url = "%s"\n' "$url"
      printf 'header = "Authorization: Bearer %s"\n' "$access_token"
      for extra_header in "$@"; do
        printf 'header = "%s"\n' "$extra_header"
      done
    } | curl -sS -o /dev/null -w '%{http_code}' -K - 2>/dev/null
  )"

  case "$status" in
    200) record "$name" "OK" "HTTP 200" ;;
    401 | 403) record "$name" "FAIL" "HTTP $status - scope present but access denied; check the API is enabled on the Cloud project and the user has access" ;;
    404) record "$name" "FAIL" "HTTP 404 - endpoint or API version not found (a sunset version is served as a generic 404)" ;;
    000) record "$name" "FAIL" "no response - network or TLS failure" ;;
    *) record "$name" "FAIL" "HTTP $status" ;;
  esac
}

# Google Ads also requires a developer token, which is account-level rather than
# credential-level, so its absence is reported separately from the probe.
ads_version="${GOOGLE_ADS_API_VERSION:-v25}"
if [[ -z "${GOOGLE_ADS_DEVELOPER_TOKEN:-}" ]]; then
  record "Google Ads API" "SKIP" "GOOGLE_ADS_DEVELOPER_TOKEN is unset"
else
  probe "Google Ads API" "https://www.googleapis.com/auth/adwords" \
    "https://googleads.googleapis.com/${ads_version}/customers:listAccessibleCustomers" \
    "developer-token: ${GOOGLE_ADS_DEVELOPER_TOKEN}"
fi

probe "GA4 Admin API" "https://www.googleapis.com/auth/analytics.readonly" \
  "https://analyticsadmin.googleapis.com/v1beta/accountSummaries?pageSize=1"

probe "Search Console API" "https://www.googleapis.com/auth/webmasters" \
  "https://searchconsole.googleapis.com/webmasters/v3/sites"

probe "Merchant / Content API" "https://www.googleapis.com/auth/content" \
  "https://shoppingcontent.googleapis.com/content/v2.1/accounts/authinfo"

# GA4's Data API has no no-argument read: every report needs a property id. The
# Admin probe above already proves the analytics half of the credential works,
# so this reports the configuration gap instead of inventing a property.
if [[ -n "${GA4_PROPERTY_ID:-}" ]]; then
  record "GA4 property id" "OK" "GA4_PROPERTY_ID is set"
else
  record "GA4 property id" "WARN" "GA4_PROPERTY_ID is unset - Data API reports take it per call"
fi

# --- report ----------------------------------------------------------------
if [[ "$json_mode" -eq 1 ]]; then
  printf '{"checks":['
  first=1
  for row in "${results[@]}"; do
    IFS=$'\t' read -r name status detail <<<"$row"
    [[ "$first" -eq 0 ]] && printf ','
    first=0
    # Only quotes and backslashes can appear unescaped in these strings; every
    # value is either a literal from this script or an HTTP status.
    esc_detail="${detail//\\/\\\\}"
    esc_detail="${esc_detail//\"/\\\"}"
    printf '{"check":"%s","status":"%s","detail":"%s"}' "$name" "$status" "$esc_detail"
  done
  printf '],"ok":%s}\n' "$([[ "$failed" -eq 0 ]] && echo true || echo false)"
else
  echo
  echo "Google ADC doctor"
  echo "================="
  for row in "${results[@]}"; do
    IFS=$'\t' read -r name status detail <<<"$row"
    printf '%-6s %-24s %s\n' "[$status]" "$name" "$detail"
  done
  echo
  if [[ "$failed" -eq 0 ]]; then
    echo "No failures. WARN rows are configuration gaps, not breakage."
  else
    echo "At least one check FAILED. Fix the topmost failure first - the later"
    echo "checks depend on the earlier ones and will keep failing until it clears."
  fi
fi

exit "$failed"
