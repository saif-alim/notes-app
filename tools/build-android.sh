#!/usr/bin/env bash
# Build platform-core as a shared library for Android ABIs.
#
# Prerequisites:
#   cargo install cargo-ndk
#   export ANDROID_NDK_HOME=/path/to/ndk  (or set via Android SDK)
#
# Usage:
#   ./tools/build-android.sh                          # default OUT_DIR
#   OUT_DIR=apps/android/app/src/main/jniLibs ./tools/build-android.sh
#
# The script cross-compiles for arm64-v8a and x86_64 (Android emulator).
# Output layout:
#   <OUT_DIR>/arm64-v8a/libplatform_core.so
#   <OUT_DIR>/x86_64/libplatform_core.so

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
OUT_DIR="${OUT_DIR:-${REPO_ROOT}/apps/android/app/src/main/jniLibs}"

cd "${REPO_ROOT}/libs/platform-core"

echo "Building platform-core for Android (arm64-v8a, x86_64)…"

cargo ndk \
    --target aarch64-linux-android \
    --target x86_64-linux-android \
    --output-dir "${OUT_DIR}" \
    build --release --features ffi

echo "Shared libraries written to ${OUT_DIR}/"
ls -lh "${OUT_DIR}/arm64-v8a/libplatform_core.so" \
       "${OUT_DIR}/x86_64/libplatform_core.so" 2>/dev/null || true
