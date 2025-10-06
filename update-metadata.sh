#!/bin/bash
#
# > make a file executable
# chmod +x ./update-metadata.sh
#
# > subxt-cli must be installed to update metadata
# cargo install subxt-cli --force
#
# Relay Chains
# subxt metadata --url wss://westend.rpc.turboflakes.io:443 -f bytes > metadata/westend_metadata.scale
subxt metadata --url wss://westend.rpc.turboflakes.io:443 --pallets System,Session,Balances,Staking,StakingAhClient,Utility,NominationPools,RcMigrator -f bytes > metadata/westend_metadata_small.scale
subxt metadata --url wss://kusama.rpc.turboflakes.io:443 -f bytes > metadata/kusama_metadata.scale
subxt metadata --url wss://kusama.rpc.turboflakes.io:443 --pallets System,Session,Balances,Staking,StakingAhClient,Utility,NominationPools,RcMigrator -f bytes > metadata/kusama_metadata_small.scale
#subxt metadata --url wss://polkadot.rpc.turboflakes.io:443 -f bytes > metadata/polkadot_metadata.scale
subxt metadata --url wss://polkadot.rpc.turboflakes.io:443 --pallets System,Session,Balances,Staking,Utility,NominationPools,RcMigrator -f bytes > metadata/polkadot_metadata_small.scale
#subxt metadata --url wss://paseo.rpc.turboflakes.io:443 -f bytes > metadata/paseo_metadata.scale
subxt metadata --url wss://paseo.rpc.turboflakes.io:443 --pallets System,Session,Balances,Staking,StakingAhClient,Utility,NominationPools,RcMigrator -f bytes > metadata/paseo_metadata_small.scale

# AssetHub Chains
subxt metadata --url wss://asset-hub-westend.rpc.turboflakes.io:443 --pallets System,Session,Balances,Staking,Utility,NominationPools,Identity -f bytes > metadata/asset_hub_westend_metadata_small.scale
subxt metadata --url wss://asset-hub-paseo.rpc.turboflakes.io:443 --pallets System,Session,Balances,Staking,Utility,NominationPools,Identity -f bytes > metadata/asset_hub_paseo_metadata_small.scale
subxt metadata --url wss://asset-hub-kusama.rpc.turboflakes.io:443 --pallets System,Session,Balances,Staking,Utility,NominationPools,Identity -f bytes > metadata/asset_hub_kusama_metadata_small.scale

# People Chains
subxt metadata --url wss://people-westend.rpc.turboflakes.io:443 --pallets Identity -f bytes > metadata/people_westend_metadata_small.scale
subxt metadata --url wss://people-kusama.rpc.turboflakes.io:443 --pallets Identity -f bytes > metadata/people_kusama_metadata_small.scale
subxt metadata --url wss://people-polkadot.rpc.turboflakes.io:443 --pallets Identity -f bytes > metadata/people_polkadot_metadata_small.scale
subxt metadata --url wss://people-paseo.rpc.turboflakes.io:443 --pallets Identity -f bytes > metadata/people_paseo_metadata_small.scale
