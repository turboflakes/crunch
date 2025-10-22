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
# subxt metadata --url wss://polkadot.rpc.turboflakes.io:443 --pallets System,Session,Balances,Staking,Utility,NominationPools,RcMigrator -f bytes > packages/chains/relay-chain-polkadot/metadata/polkadot_metadata_small.scale
subxt metadata --url wss://rpc.turboflakes.io:443/polkadot --pallets System,Session,Balances,Staking,Utility,NominationPools -f bytes > packages/chains/relay-chain-polkadot/metadata/polkadot_metadata_small_1.scale

# AssetHub Chains
subxt metadata --url wss://asset-hub-westend.rpc.turboflakes.io:443 --pallets System,Balances,Staking,Utility,NominationPools -f bytes > packages/chains/asset-hub-westend/metadata/asset_hub_westend_metadata_small.scale
subxt metadata --url wss://asset-hub-paseo.rpc.turboflakes.io:443 --pallets System,Balances,Staking,Utility,NominationPools -f bytes > packages/chains/asset-hub-paseo/metadata/asset_hub_paseo_metadata_small.scale
subxt metadata --url wss://asset-hub-kusama.rpc.turboflakes.io:443 --pallets System,Balances,Staking,Utility,NominationPools -f bytes > packages/chains/asset-hub-kusama/metadata/asset_hub_kusama_metadata_small.scale

# People Chains
subxt metadata --url wss://people-westend.rpc.turboflakes.io:443 --pallets Identity -f bytes > packages/chains/people-westend/metadata/people_westend_metadata_small.scale
subxt metadata --url wss://people-paseo.rpc.turboflakes.io:443 --pallets Identity -f bytes > packages/chains/people-paseo/metadata/people_paseo_metadata_small.scale
subxt metadata --url wss://people-kusama.rpc.turboflakes.io:443 --pallets Identity -f bytes > packages/chains/people-kusama/metadata/people_kusama_metadata_small.scale
subxt metadata --url wss://people-polkadot.rpc.turboflakes.io:443 --pallets Identity -f bytes > packages/chains/people-polkadot/metadata/people_polkadot_metadata_small.scale
