## Supported Runtimes
  - Polkadot
  - Kusama

TODO: Improve the runtimes implementation without the need of replicating the same functions for each runtime. Note that *RuntimeApi* is runtime specific. It gives access to api functions specific for each runtime.

## Generated files from subxt-cli

Download metadata from a substrate node, for use with `subxt` codegen.

```bash
# Relay Chains
subxt metadata --url wss://rpc.turboflakes.io:443/westend -f bytes > metadata/westend_metadata.scale
subxt metadata --url wss://rpc.turboflakes.io:443/westend --pallets System,Session,Balances,Staking,Utility,NominationPools -f bytes > metadata/westend_metadata_small.scale
subxt metadata --url wss://rpc.turboflakes.io:443/kusama -f bytes > metadata/kusama_metadata.scale
subxt metadata --url wss://rpc.turboflakes.io:443/kusama --pallets System,Session,Balances,Staking,Utility,NominationPools -f bytes > metadata/kusama_metadata_small.scale
subxt metadata --url wss://rpc.turboflakes.io:443/polkadot -f bytes > metadata/polkadot_metadata.scale
subxt metadata --url wss://rpc.turboflakes.io:443/polkadot --pallets System,Session,Balances,Staking,Utility,NominationPools,Identity -f bytes > metadata/polkadot_metadata_small.scale
subxt metadata --url wss://rpc.turboflakes.io:443/paseo -f bytes > paseo_metadata.scale
subxt metadata --url wss://rpc.turboflakes.io:443/paseo --pallets System,Session,Balances,Staking,Utility,NominationPools,Identity -f bytes > metadata/paseo_metadata_small.scale
# People Chains
subxt metadata --url wss://sys.turboflakes.io:443/people-westend --pallets Identity -f bytes > metadata/people_westend_metadata_small.scale
subxt metadata --url wss://sys.turboflakes.io:443/people-kusama --pallets Identity -f bytes > metadata/people_kusama_metadata_small.scale
```

Generate runtime API client code from metadata.

```bash
subxt codegen --url wss://rpc.turboflakes.io:443/westend | rustfmt --edition=2018 --emit=stdout > westend_metadata.rs
subxt codegen --url wss://rpc.turboflakes.io:443/kusama | rustfmt --edition=2018 --emit=stdout > kusama_runtime.rs
subxt codegen --url wss://rpc.turboflakes.io:443/polkadot | rustfmt --edition=2018 --emit=stdout > polkadot_runtime.rs
subxt codegen --url wss://rpc.turboflakes.io:443/kusama | rustfmt --edition=2018 --emit=stdout > kusama_runtime.rs
subxt codegen --url wss://sys.turboflakes.io:443/people-kusama | rustfmt --edition=2018 --emit=stdout > people_kusama_runtime.rs
```

Generate relay chain specs from subxt to be used under lightclient

```bash
cargo run --features chain-spec-pruning --bin subxt chain-spec --url wss://rpc.turboflakes.io:443/westend --output-file artifacts/demo_chain_specs/westend.json --state-root-hash --remove-substitutes
cargo run --features chain-spec-pruning --bin subxt chain-spec --url wss://rpc.turboflakes.io:443/kusama --output-file artifacts/demo_chain_specs/kusama.json --state-root-hash --remove-substitutes
cargo run --features chain-spec-pruning --bin subxt chain-spec --url wss://rpc.turboflakes.io:443/polkadot --output-file artifacts/demo_chain_specs/polkadot.json --state-root-hash --remove-substitutes
cargo run --features chain-spec-pruning --bin subxt chain-spec --url wss://rpc.turboflakes.io:443/paseo --output-file artifacts/demo_chain_specs/paseo.json --state-root-hash --remove-substitutes
```