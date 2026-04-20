#!/usr/bin/env bash
set -euo pipefail

# Generate openapi.json directly from Rust code without running the server
cargo run --bin gen_openapi > frontend/openapi.json
cd frontend && npx openapi-typescript ./openapi.json -o ./src/lib/generated/api-types.ts