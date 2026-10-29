#!/usr/bin/env bash
#
# Audit every skill in a skill library against the mechanical invariants that
# silently break skills: the 20,000-byte compaction cap, frontmatter whose name
# disagrees with its directory, unresolvable cross-references, reference files an
# agent can only read partially, and references nested more than one level deep.
#
# Usage:
#   bash scripts/audit-skills.sh [LIBRARY_DIR]
#   bash scripts/audit-skills.sh --help
#
# LIBRARY_DIR is the directory whose immediate children are skill directories.
# With no argument the script infers it from its own location (the library that
# contains this skill) and falls back to ~/.claude/skills. Nothing is hardcoded
# to one machine, so the same copy works from a repo checkout and from an install.
#
# Exit codes: 0 when no ERROR was reported, 1 otherwise. WARN findings never fail
# the run — they are judgment calls a human has to settle.
#
# `set -e` is deliberately omitted. An auditor must complete every check and
# report the whole picture; aborting on the first bad skill hides the rest, which
# defeats the purpose. Failures accumulate in `errors` and surface via the exit
# code.
set -uo pipefail

readonly BYTE_CAP=20000
readonly DESCRIPTION_SOFT_LIMIT=700
readonly DESCRIPTION_HARD_LIMIT=1024
readonly TOC_LINE_THRESHOLD=100

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"

errors=0
warnings=0
skills_checked=0

# Report an invariant violation. Errors fail the run.
err() {
    printf '  ERROR  %s\n' "$*"
    errors=$((errors + 1))
}

# Report a judgment call for a human to settle. Warnings never fail the run.
warn() {
    printf '  WARN   %s\n' "$*"
    warnings=$((warnings + 1))
}

usage() {
    sed -n '3,22p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
}

# True when DIR looks like a skill library: at least one DIR/*/SKILL.md exists.
looks_like_library() {
    compgen -G "$1/*/SKILL.md" >/dev/null 2>&1
}

# Print the library directory to audit, or fail when none can be determined.
resolve_library_dir() {
    local explicit="${1:-}"
    if [ -n "$explicit" ]; then
        printf '%s\n' "$explicit"
        return 0
    fi

    # scripts/ -> this skill's root -> the library root containing all skills.
    local inferred
    if inferred="$(cd -- "$SCRIPT_DIR/../.." 2>/dev/null && pwd)" &&
        looks_like_library "$inferred"; then
        printf '%s\n' "$inferred"
        return 0
    fi

    if looks_like_library "$HOME/.claude/skills"; then
        printf '%s\n' "$HOME/.claude/skills"
        return 0
    fi

    return 1
}

# Print one frontmatter field's value, flattening `>-`, `>`, `|` block scalars
# onto a single line. Prints nothing when the file has no frontmatter or no such
# key. Only column-zero keys match, so nested mapping keys cannot be mistaken for
# top-level ones.
field_value() {
    awk -v key="$2" '
        NR == 1 { if ($0 != "---") exit; next }
        $0 == "---" { if (collecting) print out; collecting = 0; exit }
        {
            if (collecting) {
                if ($0 ~ /^[[:space:]]+[^[:space:]]/) {
                    line = $0
                    sub(/^[[:space:]]+/, "", line)
                    out = (out == "" ? line : out " " line)
                    next
                }
                print out
                collecting = 0
                exit
            }
            if (index($0, key ":") == 1) {
                val = substr($0, length(key) + 2)
                sub(/^[[:space:]]+/, "", val)
                if (val ~ /^[|>][-+]?$/) { collecting = 1; out = ""; next }
                print val
                exit
            }
        }
        END { if (collecting) print out }
    ' "$1"
}

# Print every local markdown link target in FILE, one per line: absolute URLs,
# mailto and bare anchors are dropped, and any #fragment is stripped. Resolution
# against the file's own directory is the caller's job.
local_link_targets() {
    grep -oE '\]\([^)]+\)' "$1" 2>/dev/null |
        sed -e 's/^](//' -e 's/)$//' |
        while IFS= read -r target; do
            case "$target" in
            "" | '#'* | *://* | mailto:*) continue ;;
            esac
            printf '%s\n' "${target%%#*}"
        done
}

# Flag a description that reads as a procedure. A description summarizing the
# steps becomes a shortcut agents take instead of reading the skill body, so
# ordered-action phrasing is the signal worth catching.
#
# Comma count is deliberately NOT a signal. A long comma-separated list of "when
# …" clauses is the target form, and a naive comma/"then" grep flagged three
# well-written descriptions and zero real defects when measured against the
# library. Descriptions carrying three or more "when" clauses pass regardless of
# what else they contain; the rest are matched against sequence markers only:
# "then <imperative>", a next/finally/after-that connective, ordinals or numbered
# steps, and inline arrows between words.
#
# Heuristic over prose, so it always warns and never fails the run.
description_reads_as_procedure() {
    local text="$1"
    local when_clauses
    when_clauses="$(printf '%s\n' "$text" | grep -oiE '\bwhen\b' | wc -l | tr -d '[:space:]')"
    if [ "$when_clauses" -ge 3 ]; then
        return 1
    fi
    printf '%s\n' "$text" | grep -qiE \
        -e '\bthen (run|write|commit|review|dispatch|create|add|apply|verify|test|deploy|push|publish|generate|update|check|report|refactor|implement|execute|build|validate|fix|merge|open|send|hand)\b' \
        -e '\b(next|finally|lastly|afterwards|after that)[,:]' \
        -e '\bstep [1-9]\b' \
        -e '(^|[^[:alnum:]])\(?[1-9]\)[[:space:]]' \
        -e '\bfirst\b[^.]*\b(then|second)\b' \
        -e '[[:alnum:]][[:space:]]*(->|→)[[:space:]]*[[:alnum:]]'
}

check_skill() {
    local skill_dir="$1"
    local name
    name="$(basename "$skill_dir")"
    local skill_file="$skill_dir/SKILL.md"

    if [ -z "$(ls -A "$skill_dir" 2>/dev/null)" ]; then
        err "$name/ is an empty directory — delete it or add SKILL.md"
        return
    fi

    if [ ! -f "$skill_file" ]; then
        err "$name/ has no SKILL.md"
        return
    fi

    skills_checked=$((skills_checked + 1))

    if ! printf '%s' "$name" | grep -qE '^[a-z0-9]+(-[a-z0-9]+)*$'; then
        err "$name/ is not kebab-case — rename to lowercase words joined by hyphens"
    fi

    local size
    size="$(wc -c <"$skill_file" | tr -d '[:space:]')"
    if [ "$size" -ge "$BYTE_CAP" ]; then
        err "$name/SKILL.md is $size bytes (cap: under $BYTE_CAP) — move sometimes-needed content into references/"
    fi

    if [ "$(head -n 1 "$skill_file")" != "---" ]; then
        err "$name/SKILL.md has no YAML frontmatter on line 1"
        return
    fi

    local declared_name
    declared_name="$(field_value "$skill_file" name)"
    if [ -z "$declared_name" ]; then
        err "$name/SKILL.md frontmatter has no name field"
    elif [ "$declared_name" != "$name" ]; then
        err "$name/SKILL.md declares name: $declared_name — must equal the directory name"
    fi

    local description
    description="$(field_value "$skill_file" description)"
    if [ -z "$description" ]; then
        err "$name/SKILL.md frontmatter has no description field"
    else
        local length=${#description}
        if [ "$length" -gt "$DESCRIPTION_HARD_LIMIT" ]; then
            err "$name/SKILL.md description is $length chars (spec limit: $DESCRIPTION_HARD_LIMIT)"
        elif [ "$length" -gt "$DESCRIPTION_SOFT_LIMIT" ]; then
            warn "$name/SKILL.md description is $length chars — long descriptions load into every conversation"
        fi
        if description_reads_as_procedure "$description"; then
            warn "$name/SKILL.md description reads as a procedure — state triggers and scope, not ordered steps"
        fi
    fi

    check_links "$skill_file" "$name"
    check_references "$skill_dir" "$name"
}

# Verify every local markdown link in FILE resolves to something on disk.
check_links() {
    local file="$1" name="$2" base target
    base="$(dirname "$file")"
    while IFS= read -r target; do
        [ -n "$target" ] || continue
        case "$target" in
        /*) [ -e "$target" ] || err "$name: link target does not exist: $target" ;;
        *) [ -e "$base/$target" ] || err "$name: link target does not exist: $target (from ${file#"$LIBRARY_DIR"/})" ;;
        esac
    done < <(local_link_targets "$file")
}

# Enforce the reference-file rules: exactly one level deep, and a table of
# contents on anything long enough to be read partially.
check_references() {
    local skill_dir="$1" name="$2"
    local refs_dir="$skill_dir/references"
    [ -d "$refs_dir" ] || return 0

    local nested
    while IFS= read -r nested; do
        [ -n "$nested" ] || continue
        err "$name: reference nested below references/: ${nested#"$skill_dir"/} — keep references one level deep"
    done < <(find "$refs_dir" -mindepth 2 -type f 2>/dev/null)

    local ref lines
    for ref in "$refs_dir"/*.md; do
        [ -f "$ref" ] || continue
        lines="$(wc -l <"$ref" | tr -d '[:space:]')"
        if [ "$lines" -gt "$TOC_LINE_THRESHOLD" ] && ! has_toc "$ref"; then
            err "$name: $(basename "$ref") is $lines lines with no table of contents — agents preview long files with partial reads"
        fi
        check_reference_hops "$ref" "$name"
        check_links "$ref" "$name"
    done
}

# True when a reference opens with a table of contents.
#
# Detected structurally — three or more same-file anchor links in a bulleted list
# near the top — rather than by heading text. A heading-text match would be
# locale-bound: a pt-BR reference heads its contents list "## Conteudo", which an
# English-only pattern reports as missing. The structure is the same in every
# language, so that is what gets checked. A recognised heading word is accepted
# too, for a contents list written without anchor links.
has_toc() {
    local ref="$1"
    local anchor_links
    anchor_links=$(head -n 40 "$ref" | grep -cE '^[[:space:]]*[-*][[:space:]]*\[[^]]+\]\(#')
    [ "$anchor_links" -ge 3 ] && return 0
    head -n 40 "$ref" | grep -qiE '^#+[[:space:]]*(contents|table of contents|conte[uú]do|[ií]ndice|sum[aá]rio|inhalt|sommaire|contenido)' && return 0
    return 1
}

# A reference that links to another file creates a second hop. Agents follow the
# first hop and stop, so the linked file is read partially or not at all. Name
# the sibling in prose instead of linking it.
check_reference_hops() {
    local ref="$1" name="$2" target
    while IFS= read -r target; do
        [ -n "$target" ] || continue
        case "$target" in
        *.md)
            err "$name: $(basename "$ref") links to $target — references are linked from SKILL.md only, never from each other"
            ;;
        esac
    done < <(local_link_targets "$ref")
}

main() {
    case "${1:-}" in
    -h | --help)
        usage
        return 0
        ;;
    esac

    if ! LIBRARY_DIR="$(resolve_library_dir "${1:-}")"; then
        printf 'No skill library found. Pass one explicitly: bash %s LIBRARY_DIR\n' \
            "${BASH_SOURCE[0]}" >&2
        return 2
    fi
    readonly LIBRARY_DIR

    if [ ! -d "$LIBRARY_DIR" ]; then
        printf 'Not a directory: %s\n' "$LIBRARY_DIR" >&2
        return 2
    fi

    printf 'Auditing skill library: %s\n\n' "$LIBRARY_DIR"

    local skill_dir
    for skill_dir in "$LIBRARY_DIR"/*/; do
        skill_dir="${skill_dir%/}"
        [ -d "$skill_dir" ] || continue
        check_skill "$skill_dir"
    done

    printf '\n%s skills checked, %s errors, %s warnings\n' \
        "$skills_checked" "$errors" "$warnings"

    if [ "$errors" -gt 0 ]; then
        printf 'FAIL: fix every ERROR above. Each one is an invariant an agent cannot see being broken.\n' >&2
        return 1
    fi
    printf 'PASS: every mechanical invariant holds.\n'
    return 0
}

main "$@"
