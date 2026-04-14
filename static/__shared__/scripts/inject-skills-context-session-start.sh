#!/usr/bin/env bash
set -euo pipefail

# Cursor sessionStart hook: inject a compact skill catalog into conversation context.
input_json="$(cat || true)"

declare -A seen_roots=()
declare -A seen_ids=()
declare -a workspace_roots=()
declare -a skill_ids=()

max_ids="${MAX_SKILL_IDS_IN_CONTEXT:-80}"

canonical_dir() {
  local path="$1"
  if [[ -z "$path" || ! -d "$path" ]]; then
    return 1
  fi
  (
    cd "$path" >/dev/null 2>&1 && pwd -P
  )
}

add_root() {
  local candidate="$1"
  local normalized

  normalized="$(canonical_dir "$candidate")" || return 0
  if [[ -n "${seen_roots[$normalized]:-}" ]]; then
    return 0
  fi

  seen_roots["$normalized"]=1
  workspace_roots+=("$normalized")
}

collect_ids_from_skill_root() {
  local root="$1"

  if [[ ! -d "$root" ]]; then
    return 0
  fi

  while IFS= read -r -d '' skill_file; do
    local skill_dir
    local skill_id

    if (( ${#skill_ids[@]} >= max_ids )); then
      break
    fi

    if [[ -L "$skill_file" ]]; then
      continue
    fi

    skill_dir="$(dirname "$skill_file")"
    skill_id="$(basename "$skill_dir")"

    if [[ -n "${seen_ids[$skill_id]:-}" ]]; then
      continue
    fi

    seen_ids["$skill_id"]=1
    skill_ids+=("$skill_id")
  done < <(find -P "$root" -mindepth 2 -maxdepth 2 -type f -name "SKILL.md" -print0 2>/dev/null)
}

add_root "$PWD"
add_root "${CURSOR_PROJECT_DIR:-}"

if command -v jq >/dev/null 2>&1 && [[ -n "$input_json" ]]; then
  add_root "$(printf '%s' "$input_json" | jq -r '.cwd // empty')"

  while IFS= read -r root; do
    add_root "$root"
  done < <(printf '%s' "$input_json" | jq -r '.workspace_roots[]? // empty')
fi

for root in "${workspace_roots[@]}"; do
  collect_ids_from_skill_root "$root/.cursor/skills"
  collect_ids_from_skill_root "$root/.copilot/skills"
  collect_ids_from_skill_root "$root/.claude/skills"
  collect_ids_from_skill_root "$root/static/__shared__/skills"
done

collect_ids_from_skill_root "$HOME/.cursor/skills"
collect_ids_from_skill_root "$HOME/.copilot/skills"
collect_ids_from_skill_root "$HOME/.claude/skills"

summary=""
if (( ${#skill_ids[@]} == 0 )); then
  summary="No skills were discovered at session start."
else
  mapfile -t sorted_ids < <(printf '%s\n' "${skill_ids[@]}" | LC_ALL=C sort -u)
  joined="$(printf ', %s' "${sorted_ids[@]}")"
  joined="${joined:2}"
  summary="Available skills discovered at session start: ${joined}."
fi

context_text="${summary} Load relevant SKILL.md files before implementation and tests."

if command -v jq >/dev/null 2>&1; then
  jq -cn --arg context "$context_text" '{additional_context: $context}'
else
  escaped="${context_text//\\/\\\\}"
  escaped="${escaped//\"/\\\"}"
  printf '{"additional_context":"%s"}\n' "$escaped"
fi
