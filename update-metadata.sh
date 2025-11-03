#!/bin/bash
#
# > make a file executable
# chmod +x ./update-metadata.sh
#
# > subxt-cli must be installed to update metadata
# cargo install subxt-cli --force
#
# Relay Chains
# subxt metadata --url wss://westend.rpc.turboflakes.io:443 -f bytes > packages/chains/relay-chain-westend/metadata/westend_metadata.scale
subxt metadata --url wss://westend.rpc.turboflakes.io:443 --pallets Session -f bytes > packages/chains/relay-chain-westend/metadata/westend_metadata_small.scale
# subxt metadata --url wss://paseo.rpc.turboflakes.io:443 -f bytes > packages/chains/relay-chain-paseo/metadata/paseo_metadata.scale
subxt metadata --url wss://paseo.rpc.turboflakes.io:443 --pallets Session -f bytes > packages/chains/relay-chain-paseo/metadata/paseo_metadata_small.scale

# subxt metadata --url wss://kusama.rpc.turboflakes.io:443 -f bytes > packages/chains/relay-chain-kusama/metadata/kusama_metadata.scale
subxt metadata --url wss://kusama.rpc.turboflakes.io:443 --pallets Session -f bytes > packages/chains/relay-chain-kusama/metadata/kusama_metadata_small.scale
#subxt metadata --url wss://polkadot.rpc.turboflakes.io:443 -f bytes > packages/chains/relay-chain-polkadot/metadata/polkadot_metadata.scale
subxt metadata --url wss://polkadot.rpc.turboflakes.io:443 --pallets System,Session,Balances,Staking,Utility,NominationPools,StakingAhClient,RcMigrator -f bytes > packages/chains/relay-chain-polkadot/metadata/polkadot_metadata_small.scale
subxt metadata --url wss://rpc.turboflakes.io:443/polkadot --pallets System,Session,Balances,Staking,Utility,NominationPools -f bytes > packages/chains/relay-chain-polkadot/metadata/polkadot_metadata_small.scale

# AssetHub Chains
subxt metadata --url wss://asset-hub-westend.rpc.turboflakes.io:443 --pallets System,Balances,Staking,Utility,NominationPools -f bytes > packages/chains/asset-hub-westend/metadata/asset_hub_westend_metadata_small.scale
subxt metadata --url wss://asset-hub-paseo.rpc.turboflakes.io:443 --pallets System,Balances,Staking,Utility,NominationPools -f bytes > packages/chains/asset-hub-paseo/metadata/asset_hub_paseo_metadata_small.scale
subxt metadata --url wss://asset-hub-kusama.rpc.turboflakes.io:443 --pallets System,Balances,Staking,Utility,NominationPools -f bytes > packages/chains/asset-hub-kusama/metadata/asset_hub_kusama_metadata_small.scale
subxt metadata --url wss://asset-hub-polkadot.rpc.turboflakes.io:443 --pallets System,Balances,Staking,Utility,NominationPools -f bytes > packages/chains/asset-hub-polkadot/metadata/asset_hub_polkadot_metadata_small.scale

# People Chains
subxt metadata --url wss://people-westend.rpc.turboflakes.io:443 --pallets Identity -f bytes > packages/chains/people-westend/metadata/people_westend_metadata_small.scale
subxt metadata --url wss://people-paseo.rpc.turboflakes.io:443 --pallets Identity -f bytes > packages/chains/people-paseo/metadata/people_paseo_metadata_small.scale
subxt metadata --url wss://people-kusama.rpc.turboflakes.io:443 --pallets Identity -f bytes > packages/chains/people-kusama/metadata/people_kusama_metadata_small.scale
subxt metadata --url wss://people-polkadot.rpc.turboflakes.io:443 --pallets Identity -f bytes > packages/chains/people-polkadot/metadata/people_polkadot_metadata_small.scale

# Generate runtime API client code from metadata.

# ```bash
# subxt codegen --url wss://rpc.turboflakes.io:443/westend | rustfmt --edition=2018 --emit=stdout > westend_metadata.rs
# subxt codegen --url wss://rpc.turboflakes.io:443/kusama | rustfmt --edition=2018 --emit=stdout > kusama_runtime.rs
# subxt codegen --url wss://asset-hub-paseo.rpc.turboflakes.io:443 | rustfmt --edition=2018 --emit=stdout > asset_hub_paseo_runtime.rs
# subxt codegen --url wss://paseo.rpc.turboflakes.io:443 | rustfmt --edition=2018 --emit=stdout > paseo_runtime.rs
# subxt codegen --url wss://polkadot.rpc.turboflakes.io:443 | rustfmt --edition=2018 --emit=stdout > polkadot_runtime.rs
# subxt codegen --url wss://sys.turboflakes.io:443/people-kusama | rustfmt --edition=2018 --emit=stdout > people_kusama_runtime.rs
# subxt codegen --url wss://sys.turboflakes.io:443/people-polkadot | rustfmt --edition=2018 --emit=stdout > people_polkadot_runtime.rs
# subxt codegen --url wss://asset-hub-polkadot.rpc.turboflakes.io:443 | rustfmt --edition=2018 --emit=stdout > packages/chains/asset-hub-polkadot/metadata/asset_hub_polkadot_metadata_small.rs
# ```

# Generate relay chain specs from subxt to be used under lightclient

# ```bash
# cargo run --features chain-spec-pruning --bin subxt chain-spec --url wss://rpc.turboflakes.io:443/westend --output-file artifacts/demo_chain_specs/westend.json --state-root-hash --remove-substitutes
# cargo run --features chain-spec-pruning --bin subxt chain-spec --url wss://rpc.turboflakes.io:443/kusama --output-file artifacts/demo_chain_specs/kusama.json --state-root-hash --remove-substitutes
# cargo run --features chain-spec-pruning --bin subxt chain-spec --url wss://rpc.turboflakes.io:443/polkadot --output-file artifacts/demo_chain_specs/polkadot.json --state-root-hash --remove-substitutes
# cargo run --features chain-spec-pruning --bin subxt chain-spec --url wss://rpc.turboflakes.io:443/paseo --output-file artifacts/demo_chain_specs/paseo.json --state-root-hash --remove-substitutes
# ```
