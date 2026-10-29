#!/usr/bin/env bash
#
# Authorize Application Default Credentials (ADC) with the exact scopes a set of
# Google APIs needs, then prove which scopes were actually granted.
#
# Why this exists as a script rather than a documented one-liner: the equivalent
# `gcloud auth application-default login --client-id-file=... --scopes=...`
# command spans multiple lines, and a shell that drops the line continuations
# runs `--scopes=...` as a separate command. gcloud then succeeds with its
# DEFAULT scopes. The resulting credential authenticates correctly and fails
# every Ads/Analytics/Search Console call with ACCESS_TOKEN_SCOPE_INSUFFICIENT —
# a failure that looks like a permissions problem on the account rather than a
# malformed login. The verification block at the end is the load-bearing part:
# it converts that silent failure into a non-zero exit at login time.
#
# Two more conditions this encodes, both of which cost real debugging time:
#   1. The login MUST use a Desktop OAuth client you created in the Google Cloud
#      Console. gcloud's built-in client is not allowed to request the `adwords`
#      scope, and the rejection is not reported as a scope problem.
#   2. The consent screen renders one checkbox per scope and they are not all
#      pre-ticked. An unticked box produces exactly the same symptom as (1).
#
# Usage:
#   google-adc-login.sh --profile ads
#   google-adc-login.sh --profile ads,analytics,search-console
#   google-adc-login.sh --profile all --client-id-file ~/oauth-desktop.json
#   google-adc-login.sh --scopes https://www.googleapis.com/auth/adwords
#
# Exit codes: 0 every required scope granted; 1 usage/precondition failure;
# 2 login completed but at least one requested scope was not granted.
set -euo pipefail

readonly TOKENINFO_URL="https://oauth2.googleapis.com/tokeninfo"

# Scopes gcloud grants by default. Kept in every profile so installing a
# narrower credential does not break unrelated gcloud usage that already
# depends on them.
readonly BASE_SCOPES=(
  "openid"
  "https://www.googleapis.com/auth/userinfo.email"
  "https://www.googleapis.com/auth/cloud-platform"
)

# Per-API scope sets. `analytics` deliberately requests BOTH readonly and edit:
# neither implies the other, the GA4 Data API (reporting) accepts readonly, and
# the GA4 Admin API (key events, custom dimensions) rejects readonly and needs
# edit. Requesting one of the two leaves half the analytics tooling broken.
scopes_for_profile() {
  case "$1" in
    ads)
      printf '%s\n' "https://www.googleapis.com/auth/adwords"
      ;;
    analytics)
      printf '%s\n' \
        "https://www.googleapis.com/auth/analytics.readonly" \
        "https://www.googleapis.com/auth/analytics.edit"
      ;;
    search-console)
      # `webmasters` (read-write) rather than `webmasters.readonly`, because
      # sitemap submission and URL inspection are the operations worth
      # automating and they are refused by the readonly scope.
      printf '%s\n' "https://www.googleapis.com/auth/webmasters"
      ;;
    merchant)
      # Single scope for both the legacy Content API for Shopping and the newer
      # Merchant API.
      printf '%s\n' "https://www.googleapis.com/auth/content"
      ;;
    business-profile)
      printf '%s\n' "https://www.googleapis.com/auth/business.manage"
      ;;
    *)
      return 1
      ;;
  esac
}

readonly ALL_PROFILES="ads analytics search-console merchant business-profile"

usage() {
  cat >&2 <<'USAGE'
Usage: google-adc-login.sh [options]

Options:
  --profile LIST          Comma-separated API profiles. One of, or any mix of:
                          ads, analytics, search-console, merchant,
                          business-profile, all
  --scopes LIST           Comma-separated raw scope URLs, added to the profiles.
  --client-id-file PATH   Desktop OAuth client JSON downloaded from the Google
                          Cloud Console. Defaults to $GOOGLE_OAUTH_CLIENT_FILE.
  -h, --help              Show this message.

At least one of --profile or --scopes is required.
USAGE
}

profiles=""
extra_scopes=""
client_file="${GOOGLE_OAUTH_CLIENT_FILE:-}"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --profile)
      profiles="${2:-}"
      shift 2
      ;;
    --scopes)
      extra_scopes="${2:-}"
      shift 2
      ;;
    --client-id-file)
      client_file="${2:-}"
      shift 2
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *)
      echo "ERROR: unknown argument '$1'" >&2
      usage
      exit 1
      ;;
  esac
done

if [[ -z "$profiles" && -z "$extra_scopes" ]]; then
  echo "ERROR: pass --profile and/or --scopes" >&2
  usage
  exit 1
fi

command -v gcloud >/dev/null 2>&1 || {
  echo "ERROR: gcloud not found. Install the Google Cloud CLI first." >&2
  exit 1
}

if [[ -z "$client_file" ]]; then
  cat >&2 <<'MSG'
ERROR: no OAuth client file given.

Create a Desktop OAuth client (Google Cloud Console -> APIs & Services ->
Credentials -> Create credentials -> OAuth client ID -> Desktop app), download
its JSON, then re-run with --client-id-file PATH or export
GOOGLE_OAUTH_CLIENT_FILE=PATH.

gcloud's own built-in client cannot request the adwords scope.
MSG
  exit 1
fi

[[ -f "$client_file" ]] || {
  echo "ERROR: OAuth client file not found: $client_file" >&2
  exit 1
}

# Expand --profile into scopes. `all` is expanded rather than special-cased so
# that adding a profile above automatically widens it.
requested=("${BASE_SCOPES[@]}")
if [[ -n "$profiles" ]]; then
  IFS=',' read -r -a profile_list <<<"$profiles"
  for profile in "${profile_list[@]}"; do
    profile="${profile// /}"
    [[ -z "$profile" ]] && continue
    if [[ "$profile" == "all" ]]; then
      for expanded in $ALL_PROFILES; do
        while IFS= read -r scope; do requested+=("$scope"); done < <(scopes_for_profile "$expanded")
      done
      continue
    fi
    if ! scopes_for_profile "$profile" >/dev/null 2>&1; then
      echo "ERROR: unknown profile '$profile'. Known: $ALL_PROFILES, all" >&2
      exit 1
    fi
    while IFS= read -r scope; do requested+=("$scope"); done < <(scopes_for_profile "$profile")
  done
fi

if [[ -n "$extra_scopes" ]]; then
  IFS=',' read -r -a raw_list <<<"$extra_scopes"
  for scope in "${raw_list[@]}"; do
    scope="${scope// /}"
    [[ -n "$scope" ]] && requested+=("$scope")
  done
fi

# De-duplicate while preserving order: a repeated scope is accepted by Google
# but makes the consent screen and the verification output harder to read.
unique=()
for scope in "${requested[@]}"; do
  seen=0
  for kept in ${unique[@]+"${unique[@]}"}; do
    [[ "$kept" == "$scope" ]] && seen=1 && break
  done
  [[ "$seen" -eq 0 ]] && unique+=("$scope")
done

scope_arg="$(
  IFS=,
  echo "${unique[*]}"
)"

echo "Requesting ${#unique[@]} scopes:"
printf '  - %s\n' "${unique[@]}"
echo
echo "On the Google consent screen, tick EVERY permission box."
echo "An unticked box is the most common cause of insufficient-scope errors."
echo

gcloud auth application-default login \
  --client-id-file="$client_file" \
  --scopes="$scope_arg"

echo
echo "=== Verifying what was actually granted ==="

token="$(gcloud auth application-default print-access-token)"

# tokeninfo reports the scopes attached to the credential Google issued, which
# is the only authoritative answer. Parsing is done with python3 when present
# and falls back to a tolerant sed extraction so the check still runs on a
# minimal host rather than being skipped — skipping it would restore exactly the
# silent failure this script exists to prevent.
# The access token is fed to curl through a stdin config file rather than the
# command line. A URL in argv is visible to every other user on the host via
# `ps`, and an access token there is a usable credential for its full lifetime.
response="$(printf 'url = "%s?access_token=%s"\n' "$TOKENINFO_URL" "$token" | curl -fsS -K -)"
if command -v python3 >/dev/null 2>&1; then
  granted="$(printf '%s' "$response" | python3 -c 'import json,sys; print(json.load(sys.stdin).get("scope",""))')"
else
  granted="$(printf '%s' "$response" | sed -n 's/.*"scope"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')"
fi

printf '%s\n' "$granted" | tr ' ' '\n' | sed '/^$/d;s/^/  - /'
echo

missing=0
for scope in "${unique[@]}"; do
  # `openid` is echoed back by tokeninfo inconsistently across client types and
  # grants no API access on its own, so its absence is not treated as a failure.
  [[ "$scope" == "openid" ]] && continue
  case " $granted " in
    *" $scope "*) ;;
    *)
      echo "MISSING: $scope" >&2
      missing=1
      ;;
  esac
done

if [[ "$missing" -eq 1 ]]; then
  cat >&2 <<'MSG'

FAILED - at least one requested scope was not granted. Calls needing it will
return ACCESS_TOKEN_SCOPE_INSUFFICIENT, which reads like an account permission
problem but is not one.

Re-run this script and tick every box on the consent screen. If a scope is
refused no matter what, the OAuth client is not a Desktop client, or the API it
belongs to is not enabled on the Cloud project.
MSG
  exit 2
fi

echo "SUCCESS - every requested scope was granted."
echo
echo "If MCP servers or long-lived processes read this credential, restart or"
echo "reconnect them now: they cache the previous credential in memory and will"
echo "keep failing with the old scope set."
