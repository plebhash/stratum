#!/bin/sh

# USAGE:
#   ./scripts/cargo-publish.sh <crate-dir> [--dry-run]
#
# If --dry-run is provided, the script runs cargo publish --dry-run,
# otherwise it performs a real publish.

CRATE_DIR="$1"
DRY_FLAG="${2:-}"

echo "Publishing crate in directory: $CRATE_DIR"

cd "$CRATE_DIR"

if [ "$DRY_FLAG" = "--dry-run" ]; then
  echo "Dry run enabled. Will not actually publish to crates.io."
  CARGO_COMMAND="cargo publish --dry-run"
else
  CARGO_COMMAND="cargo publish"
fi

OUTPUT="$($CARGO_COMMAND 2>&1)"
EXIT_CODE=$?
echo "Ran cargo command, exit code was $EXIT_CODE"

if [ "$DRY_FLAG" = "--dry-run" ]; then
  if [ "$EXIT_CODE" -eq 0 ]; then
    echo "Dry-run succeeded; nothing was actually published."
    exit 0
  fi
  # If it failed under --dry-run, handle error or log accordingly:
  echo "Dry-run failed for $CRATE_DIR."
  echo "$OUTPUT"
  exit 1
fi

if [ "$EXIT_CODE" -eq 0 ] ; then
  echo "Publish command succeeded: $CRATE_DIR"
  exit 0
fi

# If cargo failed, check whether it was 'already uploaded'
if echo "$OUTPUT" | grep -q "already uploaded"; then
  echo "Crate is already published: $CRATE_DIR"
  exit 0
fi

echo "Publish command failed for $CRATE_DIR"
echo "$OUTPUT"
exit 1