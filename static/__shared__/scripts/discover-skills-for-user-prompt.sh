#!/usr/bin/env bash
set -euo pipefail

# Hook contract:
# - Input: optional JSON from stdin (tool-dependent), used to discover extra roots.
# - Output: always prints {"continue":true} so prompt submission is never blocked.
# - Side effect: writes a discovery summary to stderr for observability.
# Discovery scope:
# - Workspace roots from PWD, *_PROJECT_DIR vars, and optional stdin fields.
# - User-level skills under $HOME for copilot/cursor/claude.
input_json="$(cat || true)"

have_jq=0
if command -v jq >/dev/null 2>&1; then
  have_jq=1
fi

declare -A seen_roots=()
declare -A seen_skill_keys=()
declare -a workspace_roots=()
declare -a skills=()

max_skills="${MAX_SKILLS_DISCOVERED:-5000}"

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

collect_skills_from_dir() {
  local root="$1"
  local source="$2"
  local platform="$3"

  if [[ ! -d "$root" ]]; then
    return 0
  fi

  # Enumerate <root>/<skill-id>/SKILL.md and dedupe by source/platform/id/path.
  while IFS= read -r -d '' skill_file; do
    local skill_dir
    local skill_id
    local key

    if (( ${#skills[@]} >= max_skills )); then
      echo "[skill-discovery] reached MAX_SKILLS_DISCOVERED=${max_skills}; truncating results" >&2
      break
    fi

    if [[ -L "$skill_file" ]]; then
      continue
    fi

    skill_dir="$(dirname "$skill_file")"
    skill_id="$(basename "$skill_dir")"
    key="${source}|${platform}|${skill_id}|${skill_file}"

    if [[ -n "${seen_skill_keys[$key]:-}" ]]; then
      continue
    fi

    seen_skill_keys["$key"]=1
    skills+=("${skill_id}|${source}|${platform}|${skill_file}")
  done < <(find -P "$root" -mindepth 2 -maxdepth 2 -type f -name "SKILL.md" -print0 2>/dev/null)
}

# Collect roots from common hook inputs and environment variables.
add_root "$PWD"
add_root "${CURSOR_PROJECT_DIR:-}"
add_root "${CLAUDE_PROJECT_DIR:-}"

if [[ $have_jq -eq 1 && -n "$input_json" ]]; then
  add_root "$(printf '%s' "$input_json" | jq -r '.cwd // empty')"

  while IFS= read -r root; do
    add_root "$root"
  done < <(printf '%s' "$input_json" | jq -r '.workspace_roots[]? // empty')
fi

# Discover workspace/project skills.
for root in "${workspace_roots[@]}"; do
  collect_skills_from_dir "$root/.copilot/skills" "workspace" "copilot"
  collect_skills_from_dir "$root/.cursor/skills" "workspace" "cursor"
  collect_skills_from_dir "$root/.claude/skills" "workspace" "claude"
  collect_skills_from_dir "$root/static/__shared__/skills" "workspace" "shared"
done

# Discover user-level installed skills.
collect_skills_from_dir "$HOME/.copilot/skills" "user" "copilot"
collect_skills_from_dir "$HOME/.cursor/skills" "user" "cursor"
collect_skills_from_dir "$HOME/.claude/skills" "user" "claude"

skill_count=0
if [[ ${#skills[@]} -gt 0 ]]; then
  while IFS= read -r line; do
    if [[ -n "$line" ]]; then
      skill_count=$((skill_count + 1))
    fi
  done < <(printf '%s\n' "${skills[@]}" | LC_ALL=C sort -u)
fi

echo "[skill-discovery] found ${skill_count} skills across workspace and user roots" >&2

# Keep output schema compatible with prompt-submission hooks.
printf '{"continue":true}\n'
