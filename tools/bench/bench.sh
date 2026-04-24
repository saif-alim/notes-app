#!/usr/bin/env bash
# Load smoke test: oha benchmark against local backend
# Usage: ./tools/bench/bench.sh
# Requires: oha (brew install oha)
# Backend must be running: bazel run //services/api:notes_api
set -euo pipefail

BASE_URL="${BASE_URL:-http://127.0.0.1:3000}"

echo "--- warming up ---"
oha -n 10 -c 1 -q "$BASE_URL/notes" > /dev/null

echo ""
echo "=== GET /notes — 1000 req, 20 concurrent ==="
oha -n 1000 -c 20 "$BASE_URL/notes"

echo ""
echo "=== POST /notes — 500 req, 10 concurrent ==="
oha -n 500 -c 10 -m POST \
  -H "content-type: application/json" \
  -d '{"body":"bench note"}' \
  "$BASE_URL/notes"
