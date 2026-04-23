#!/usr/bin/env bash
# Build the iOS app with Bazel and install+launch on a simulator.
# Usage: ./tools/run-ios-sim.sh [SIMULATOR_UDID]
#
# If UDID is omitted, uses the first available booted simulator or boots iPhone 16 Pro.
set -euo pipefail

BUNDLE_ID="com.notes.app"
DEFAULT_DEVICE="iPhone 16 Pro"
IPA_TMP="$(mktemp -d)/notes-ipa"

echo "==> Building //apps/ios:NotesApp ..."
bazel build //apps/ios:NotesApp

echo "==> Extracting .app from IPA ..."
mkdir -p "$IPA_TMP"
unzip -q bazel-bin/apps/ios/NotesApp.ipa -d "$IPA_TMP"
APP="$IPA_TMP/Payload/NotesApp.app"

# Determine simulator UDID
if [ -n "${1:-}" ]; then
  UDID="$1"
else
  UDID=$(xcrun simctl list devices available | grep Booted | head -1 | grep -Eo '[0-9A-F-]{36}' || true)
  if [ -z "$UDID" ]; then
    echo "==> No booted simulator found. Booting $DEFAULT_DEVICE ..."
    UDID=$(xcrun simctl list devices available | grep "$DEFAULT_DEVICE" | head -1 | grep -Eo '[0-9A-F-]{36}')
    xcrun simctl boot "$UDID"
  fi
fi

echo "==> Installing on simulator $UDID ..."
xcrun simctl install "$UDID" "$APP"

echo "==> Launching $BUNDLE_ID ..."
LAUNCH_OUTPUT=$(xcrun simctl launch "$UDID" "$BUNDLE_ID" 2>/dev/null || true)

open -a Simulator

echo "==> Done. App running as $LAUNCH_OUTPUT"
