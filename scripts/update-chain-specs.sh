#!/bin/bash
#
# > make a file executable
# chmod +x ./update-chain-specs.sh
#
# > subxt-cli must be installed to update metadata
# cargo install subxt-cli --force --features chain-spec-pruning

BASE="packages/support"

fetch_chain_specs() {
  local chain="$1"      # e.g. "westend"
  local host="$2"       # e.g. "westend.rpc.turboflakes.io"

  # Derive output filename: replace hyphens with underscores
  local filename="${chain//-/_}.json"
  local out_dir="$BASE/chain-specs"

  mkdir -p "$out_dir"

  # Retry fetching metadata up to $max_attempts times
  local max_attempts=3
  local attempt=1
  until subxt chain-spec --url wss://$host:443 --output-file "$out_dir/$filename" --state-root-hash --remove-substitutes; do
    if [ "$attempt" -ge "$max_attempts" ]; then
      echo "ERROR: failed to fetch metadata for $chain after $max_attempts attempts"
      return 1
    fi
    echo "Attempt $attempt failed for $chain, retrying in 10s..."
    attempt=$((attempt + 1))
    sleep 10
  done

}

# Relay Chains
fetch_chain_specs "westend"  "westend.rpc.turboflakes.io"
fetch_chain_specs "paseo"    "paseo.rpc.turboflakes.io"
fetch_chain_specs "kusama"   "kusama.rpc.turboflakes.io"
fetch_chain_specs "polkadot" "polkadot.rpc.turboflakes.io"

# Generate relay chain specs from subxt to be used under lightclient

# ```bash
# cargo run --features chain-spec-pruning --bin subxt chain-spec --url wss://rpc.turboflakes.io:443/westend --output-file artifacts/demo_chain_specs/westend.json --state-root-hash --remove-substitutes
# cargo run --features chain-spec-pruning --bin subxt chain-spec --url wss://rpc.turboflakes.io:443/kusama --output-file artifacts/demo_chain_specs/kusama.json --state-root-hash --remove-substitutes
# cargo run --features chain-spec-pruning --bin subxt chain-spec --url wss://rpc.turboflakes.io:443/polkadot --output-file artifacts/demo_chain_specs/polkadot.json --state-root-hash --remove-substitutes
# cargo run --features chain-spec-pruning --bin subxt chain-spec --url wss://rpc.turboflakes.io:443/paseo --output-file artifacts/demo_chain_specs/paseo.json --state-root-hash --remove-substitutes
# ```
