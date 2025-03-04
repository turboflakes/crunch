#!/bin/bash
#
# > make a file executable
# chmod +x ./update-metadata.sh
#
# > subxt-cli must be installed to update metadata
# cargo install subxt-cli --force
#
# Relay Chains
#subxt metadata --url wss://rpc.turboflakes.io:443/westend -f bytes > metadata/westend_metadata.scale
subxt metadata --url wss://rpc.turboflakes.io:443/westend --pallets System,Session,Balances,Staking,Utility,NominationPools -f bytes > metadata/westend_metadata_small.scale
#subxt metadata --url wss://rpc.turboflakes.io:443/kusama -f bytes > metadata/kusama_metadata.scale
subxt metadata --url wss://rpc.turboflakes.io:443/kusama --pallets System,Session,Balances,Staking,Utility,NominationPools -f bytes > metadata/kusama_metadata_small.scale
#subxt metadata --url wss://rpc.turboflakes.io:443/polkadot -f bytes > metadata/polkadot_metadata.scale
subxt metadata --url wss://rpc.turboflakes.io:443/polkadot --pallets System,Session,Balances,Staking,Utility,NominationPools -f bytes > metadata/polkadot_metadata_small.scale
#subxt metadata --url wss://rpc.turboflakes.io:443/paseo -f bytes > metadata/paseo_metadata.scale
subxt metadata --url wss://rpc.turboflakes.io:443/paseo --pallets System,Session,Balances,Staking,Utility,NominationPools,Identity -f bytes > metadata/paseo_metadata_small.scale
# People Chains
subxt metadata --url wss://sys.turboflakes.io:443/people-westend --pallets Identity -f bytes > metadata/people_westend_metadata_small.scale
subxt metadata --url wss://sys.turboflakes.io:443/people-kusama --pallets Identity -f bytes > metadata/people_kusama_metadata_small.scale
subxt metadata --url wss://sys.turboflakes.io:443/people-polkadot --pallets Identity -f bytes > metadata/people_polkadot_metadata_small.scale
subxt metadata --url wss://sys.turboflakes.io:443/people-paseo --pallets Identity -f bytes > metadata/people_paseo_metadata_small.scale
