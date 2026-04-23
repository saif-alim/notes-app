#!/usr/bin/env bash
# Non-blocking advisory Stop hook.
# Warns if code changed without matching doc update in same change set.
set -euo pipefail

cd "$(git rev-parse --show-toplevel 2>/dev/null)" || exit 0

changed=$(git diff --name-only HEAD 2>/dev/null; git diff --name-only --cached 2>/dev/null)
[ -z "$(printf %s "$changed" | tr -d '[:space:]')" ] && exit 0

code=0; docs=0
while IFS= read -r f; do
  [ -z "$f" ] && continue
  case "$f" in
    *.md|*.MD) docs=1 ;;
    *.rs|*.swift|*.proto|*.bzl|*.bazel|MODULE.bazel|BUILD.bazel|WORKSPACE|.bazelrc) code=1 ;;
  esac
done <<< "$changed"

if [ "$code" = 1 ] && [ "$docs" = 0 ]; then
  echo "[docs-freshness] Code changed, no *.md updated. Verify README/architecture/test-plan/retrospective/area CLAUDE.md still describe reality. If genuinely unaffected, note so in commit body." >&2
fi
exit 0
