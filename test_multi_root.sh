#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "${SCRIPT_DIR}"

# Test multi-root grounding
echo "Testing multi-root grounding..."
cargo run -p invariant-path-cli -- scan --file ./README.adoc --artifact-uri "repo://README.adoc" --json > /tmp/root1.json
cargo run -p invariant-path-cli -- scan --file ./crates/invariant-path-core/src/lib.rs --artifact-uri "repo://core/lib.rs" --json > /tmp/root2.json

echo "Multi-root grounding test completed successfully!"
echo "Annotations from root1: $(jq 'length' /tmp/root1.json)"
echo "Annotations from root2: $(jq 'length' /tmp/root2.json)"