#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# Verify that the correspondence record names the exact Agda proof bytes.
set -euo pipefail

record="examples/same-cube/correspondence.json"
proof="proofs/SameCube.agda"

for tool in jq git sha256sum; do
  command -v "$tool" >/dev/null || { echo "ERROR: missing $tool" >&2; exit 1; }
done

recorded_path=$(jq -er '.witness.source_path' "$record")
recorded_git_blob=$(jq -er '.witness.source_git_blob_sha1' "$record")
recorded_sha256=$(jq -er '.witness.source_sha256' "$record")

if [ "$recorded_path" != "$proof" ]; then
  echo "ERROR: unexpected witness source path in $record" >&2
  exit 1
fi

actual_git_blob=$(git hash-object "$proof")
actual_sha256=$(sha256sum "$proof" | cut -d' ' -f1)

if [ "$recorded_git_blob" != "$actual_git_blob" ]; then
  echo "ERROR: Agda Git blob does not match the correspondence record" >&2
  exit 1
fi
if [ "$recorded_sha256" != "$actual_sha256" ]; then
  echo "ERROR: Agda SHA-256 does not match the correspondence record" >&2
  exit 1
fi

echo "Proof witness digests match the recorded Agda source."
