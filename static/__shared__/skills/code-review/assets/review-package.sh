#!/usr/bin/env bash
# Write a review package — commit list, stat summary, and the net diff with
# wide context — to a file outside the repository, so the reviewer reads the
# whole change in one Read call and the diff never enters the coordinator's
# context window.
#
# Two properties are load-bearing:
#   * -U10 context. It lets the reviewer judge hunks without opening the
#     changed files, which is what makes a diff-scoped reading budget
#     survivable.
#   * The SHA range in the filename. A re-review after fixes gets a distinct
#     fresh file instead of silently reading a stale package.
#
# Read-only by construction: every git invocation here only inspects history
# (log, diff, rev-parse, rev-list, merge-base). Nothing touches the working
# tree, the index, HEAD, or branch state, and the output is written under
# TMPDIR rather than into the checkout under review.
#
# Usage: bash review-package.sh BASE HEAD [OUTFILE]
# Prints the path it wrote. Exit 2 on bad usage or unresolvable refs, exit 3
# on a range that cannot describe the change under review.
#
# Adapted from obra/superpowers (MIT, (c) 2025 Jesse Vincent).

set -euo pipefail

if [ $# -lt 2 ] || [ $# -gt 3 ]; then
  echo "usage: bash review-package.sh BASE HEAD [OUTFILE]" >&2
  exit 2
fi

base=$1
head=$2

git rev-parse --git-dir >/dev/null 2>&1 || { echo "not a git repository" >&2; exit 2; }
git rev-parse --verify --quiet "${base}^{commit}" >/dev/null || { echo "bad BASE: $base" >&2; exit 2; }
git rev-parse --verify --quiet "${head}^{commit}" >/dev/null || { echo "bad HEAD: $head" >&2; exit 2; }

# Range guards. A wrong-branch or stale HEAD yields a range that is either
# empty or not rooted at BASE; both silently produce a package that does not
# describe the change being reviewed, so fail loudly rather than hand the
# reviewer a plausible-looking lie.
git merge-base --is-ancestor "$base" "$head" || {
  echo "HEAD is not a descendant of BASE: ${base}..${head}" >&2
  exit 3
}
commits=$(git rev-list --count "${base}..${head}")
[ "$commits" -gt 0 ] || {
  echo "empty commit range: ${base}..${head}" >&2
  exit 3
}

if [ $# -eq 3 ]; then
  out=$3
else
  dir="${TMPDIR:-/tmp}/code-review"
  mkdir -p "$dir"
  out="${dir}/review-$(git rev-parse --short "$base")..$(git rev-parse --short "$head").diff"
fi

{
  echo "# Review package: ${base}..${head}"
  echo
  echo "## Commits"
  git log --oneline "${base}..${head}"
  echo
  echo "## Files changed"
  git diff --stat "${base}..${head}"
  echo
  echo "## Diff"
  git diff -U10 "${base}..${head}"
} > "$out"

printf 'wrote %s: %s commit(s), %s bytes\n' \
  "$out" "$commits" "$(wc -c < "$out" | tr -d ' ')"
