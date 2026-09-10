#!/bin/bash
#
# Prints the spec versions embedded in the committed runtime metadata files
# under packages/chains/*/artifacts/metadata, grouped by network.
#
# Optionally flag specific chains as updated by passing their names as
# arguments, e.g. `print-metadata-versions.sh westend asset-hub-westend`.
#
# > subxt-cli must be installed
# cargo install subxt-cli --force
BASE="packages/chains"

spec_version_of() {
  local chain="$1"      # e.g. "westend", "asset-hub-westend", "people-westend"

  local tmp="${chain//relay-chain-/}"
  local filename="${tmp//-/_}_metadata_small.scale"
  local file="$BASE/$chain/metadata/$filename"

  if [ ! -f "$file" ]; then
    echo "unknown"
    return
  fi

  subxt explore --file "$file" pallet System constants Version 2>/dev/null \
    | sed -r 's/\x1b\[[0-9;]*m//g' \
    | awk '/value of the constant is/{f=1} f' \
    | grep -oE 'spec_version: [0-9]+' | grep -oE '[0-9]+'
}

is_updated() {
  local chain="$1"
  shift
  local updated
  for updated in "$@"; do
    [ "$updated" = "$chain" ] && return 0
  done
  return 1
}

echo "📦️ Current builtin runtime metadata"
for chain in relay-chain-polkadot asset-hub-polkadot people-polkadot \
             relay-chain-kusama asset-hub-kusama people-kusama \
             relay-chain-paseo asset-hub-paseo people-paseo \
             relay-chain-westend asset-hub-westend people-westend; do
  suffix=""
  is_updated "$chain" "$@" && suffix=" (updated)"
  echo "- $chain/$(spec_version_of "$chain")$suffix"
done
