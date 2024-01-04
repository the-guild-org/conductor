#!/bin/bash

# Paths of the files
OURS="$2"
THEIRS="$3"
MERGED="$1"

# Always choose THEIRS (incoming changes)
cp "$THEIRS" "$MERGED"

# Perform a cargo check
cargo check
if [ $? -eq 0 ]; then
  # If cargo check succeeds, exit with success
  exit 0
else
  # If cargo check fails, exit with error
  echo "Cargo check failed, manual resolution may be required."
  exit 1
fi
