#!/usr/bin/env bash
#
# find-polluter.sh — identify which test file leaves unwanted state behind.
#
# Intent
#   Some bugs are not in the failing test; they are in a different test that
#   created a file, directory, database, or global that the failing test then
#   inherits. Run order makes the symptom move, which is why these are usually
#   misfiled as flakiness. This script removes the guessing: it runs the matching
#   test files one at a time, checking after each whether the polluting artifact
#   appeared, and stops at the first file that produced it.
#
#   It is the bisection step referenced by
#   systematic-debugging/references/root-cause-tracing.md. The polluter it names
#   is the place to start Phase 1 investigation, not the place to add a cleanup
#   call — deleting the artifact in an afterEach hides the bug.
#
# Usage
#   bash find-polluter.sh <path_to_check> <test_file_glob> [test_command]
#
#   path_to_check     File or directory whose appearance marks pollution,
#                     relative to the current directory. Example: '.git'
#   test_file_glob    Glob matching the test files to bisect, quoted so the
#                     shell does not expand it. Example: 'src/**/*.test.ts'
#   test_command      Command that runs a single test file; the file path is
#                     appended as its last argument. Defaults to 'npm test'.
#                     Examples: 'npm test', 'npx vitest run', 'pytest -q',
#                     'go test -run . -count=1', 'cargo test --test'
#
# Examples
#   bash find-polluter.sh '.git' 'src/**/*.test.ts' 'npx vitest run'
#   bash find-polluter.sh 'tmp/fixtures.db' 'tests/**/*_test.py' 'pytest -q'
#
# Exit codes
#   0  No polluter found: every matched file ran without producing the artifact.
#   1  Polluter found (its path is printed), or the arguments were invalid.
#
# Failure modes to know before trusting the result
#   * The artifact already exists when the run starts. The script reports this
#     and skips, because it cannot attribute an artifact it did not watch appear.
#     Remove the artifact first, then re-run.
#   * The polluter only pollutes in combination with another test, or only when
#     the suite runs in parallel. One-file-at-a-time bisection will not find it;
#     fall back to tracing the artifact's creation site directly.
#   * The test command cannot run a single file (some suites only run whole
#     packages). Then each step runs more than one file and the result narrows to
#     a package, not a file.
#   * Test failures are ignored on purpose: a test that errors can still pollute,
#     and its exit code is not the signal being measured here.
#
# Run from the repository root: both the glob and the checked path are resolved
# relative to the current directory.

set -euo pipefail

usage() {
  echo "Usage: $0 <path_to_check> <test_file_glob> [test_command]"
  echo "Example: $0 '.git' 'src/**/*.test.ts' 'npx vitest run'"
}

if [ $# -lt 2 ] || [ $# -gt 3 ]; then
  usage
  exit 1
fi

POLLUTION_CHECK="$1"
TEST_PATTERN="$2"
TEST_COMMAND="${3:-npm test}"

echo "Searching for the test that creates: $POLLUTION_CHECK"
echo "Test pattern:  $TEST_PATTERN"
echo "Test command:  $TEST_COMMAND <file>"
echo ""

# `find .` emits ./-prefixed paths, so accept the pattern written either way.
TEST_PATTERN="${TEST_PATTERN#./}"

# `find -path` matches '**/' against one or more directory levels, never zero,
# so a pattern like src/**/*.test.ts would silently skip src/top.test.ts. Also
# try the pattern with '**/' collapsed to cover files directly under the base.
TEST_FILES=$(find . \( -path "./$TEST_PATTERN" -o -path "./${TEST_PATTERN//\*\*\//}" \) | sort -u)

if [ -z "$TEST_FILES" ]; then
  echo "No test files matched '$TEST_PATTERN'."
  echo "Check the glob and that you are running from the repository root."
  exit 1
fi

TOTAL=$(printf '%s\n' "$TEST_FILES" | wc -l | tr -d ' ')
echo "Found $TOTAL test files"
echo ""

COUNT=0
for TEST_FILE in $TEST_FILES; do
  COUNT=$((COUNT + 1))

  # Attribution requires watching the artifact appear. If it is already present,
  # this step proves nothing about this file.
  if [ -e "$POLLUTION_CHECK" ]; then
    echo "Pollution already present before step $COUNT/$TOTAL."
    echo "  Remove '$POLLUTION_CHECK' and re-run; skipping: $TEST_FILE"
    continue
  fi

  echo "[$COUNT/$TOTAL] $TEST_FILE"

  # A failing test can still pollute, so the exit status is deliberately ignored.
  $TEST_COMMAND "$TEST_FILE" > /dev/null 2>&1 || true

  if [ -e "$POLLUTION_CHECK" ]; then
    echo ""
    echo "FOUND POLLUTER"
    echo "  Test:    $TEST_FILE"
    echo "  Created: $POLLUTION_CHECK"
    echo ""
    ls -la "$POLLUTION_CHECK"
    echo ""
    echo "Next steps:"
    echo "  $TEST_COMMAND $TEST_FILE   # reproduce in isolation"
    echo "  Trace where the artifact is created, then fix at that source."
    echo "  Do not add cleanup to hide it; cleanup leaves the bug in place."
    exit 1
  fi
done

echo ""
echo "No polluter found across $TOTAL files run individually."
echo "If the artifact still appears in a full run, the cause needs two or more"
echo "tests together, or parallel execution; trace its creation site instead."
exit 0
