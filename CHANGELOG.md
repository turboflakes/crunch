# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.26.1] - 2025-10-23

## New
- The value of CRUNCH_MAXIMUM_CALLS is DEPRECATED in favour of optimizing the number of calls that fit in a single batch call. Crunch recursively validates the weight of the maximum number of pending payouts and caps the number of calls as defined in CRUNCH_MAXIMUM_PAYOUTS.

## Changed
- Fix issue #65 & #66 where CRUNCH_SUBSTRATE_PEOPLE_WS_URL is always optional and do not need to be set
- Fix issue #46 project structure has been split into several packages to optimize compilation times
- Fix nomination pools API for Kusama, Paseo and Westend
- Fix gracefully handle runtime upgrades

- Update metadata polkadot/1007001
- Update metadata people-polkadot/1007001
- Update metadata kusama/1009002
- Update metadata asset-hub-kusama/1009002
- Update metadata people-kusama/1009001
- Update metadata paseo/1009002
- Update metadata asset-hub-paseo/1009002
- Update metadata people-paseo/1009002
- Update metadata westend/1020001
- Update metadata asset-hub-westend/1020003
- Update metadata people-westend/1020001

## [0.25.0] - 2025-10-06

- Support AHM on Kusama
- Update subxt to `v0.44.0`
- Update metadata polkadot/1006002
- Update metadata people-polkadot/1006001
- Update metadata kusama/1009001
- Update metadata asset-hub-kusama/1009001
- Update metadata people-kusama/1009001
- Update metadata paseo/1006002
- Update metadata asset-hub-paseo/1006002
- Update metadata people-paseo/1007001
- Update metadata westend/1020001
- Update metadata asset-hub-westend/1020001
- Update metadata people-westend/1020001

## [0.24.2] - 2025-10-03

## Changed
- Fix issue on querying the validator active status introduced in previous release on Westend and Paseo chains.
- Fix breaking change introduced in previous release where the `chain` required to be present in the CLI. Note that the `chain` in CLI is only mandatory when in combination with unstable `--enable-light-client` feature.
- Add validity check on all genesis state roots hashes for all chains
- Add env `CRUNCH_MAXIMUM_ERROR_INTERVAL` allows crunch to restart operations in case of RPC connection errors every 24 hours as default.
- SPECIAL NOTE: To prevent unnecessary operations during asset hub migration a sanity check will be introduced on crunch supporting chains KUSAMA and POLKADOT in future releases.
Currently is only available on PASEO: Add env `CRUNCH_SANITY_SLEEP_INTERVAL` which allows crunch to check asset hub migration stage, default is to check stage every 10 minutes. The logic is as follows: If AHM is in progress just hold operations and try again later after sleep interval. If AHM is scheduled or pending, continue operations on RC. If AHM is completed, continue operations on AH. This is valid for all crunch execution modes: `era, daily, turbo, once`;
- Update metadata polkadot/1007001
- Update metadata people-polkadot/1007001


## [0.24.1] - 2025-09-23

## Changed
- Add flag `--substrate-asset-hub-ws-url` to set a custom AssetHub RPC endpoint

## [0.24.0] - 2025-09-23

## Changed
- Support staking on asset-hub test networks (Paseo, Westend) and deprecate from respective relay chains
- Add Dockerfile.alpine example
- Update all chain-specs required for lightclient use
- Update subxt to `v0.43.0`
- Update metadata polkadot/1006002
- Update metadata people-polkadot/1006001
- Update metadata kusama/1007001
- Update metadata people-kusama/1007001
- Update metadata asset-hub-paseo/1006002
- Update metadata people-paseo/1007001
- Update metadata asset-hub-westend/1020000
- Update metadata people-westend/1020000

## [0.23.0] - 2025-05-26

## Changed
- Fix light client support
- Fix #61 Polkadot, Paseo, Westend subscription support

## [0.22.1] - 2025-05-26

## Changed
- Fix light-client support
- Fix Polkadot, Paseo and Kusama subscription support
- Update metadata polkadot/1005001
- Update metadata people-polkadot/1005001


## [0.22.0] - 2025-05-26

## Changed
- Disable light-client feature (to be available in an upcoming release)
- Update subxt to `v0.42.1`
- Update metadata polkadot/1004001
- Update metadata people-polkadot/1004000
- Update metadata people-kusama/1005001
- Update metadata kusama/1005001
- Update metadata people-kusama/1005001
- Update metadata paseo/1004003
- Update metadata people-paseo/1004003
- Update metadata westend/1018005
- Update metadata people-westend/1018000

## [0.21.0] - 2025-04-08

## Changed
- Fix Dockerfile example
- Fix matrix `home_server` missing field from authentication API
- Update all chains specs to be used under lightclient feature
- Update metadata paseo/1004001
- Update metadata people-paseo/1004001
- Update metadata westend/1018001
- Update metadata people-westend/1018000

## [0.20.0] - 2025-03-10

## Changed
- Crunch binary available for multiple releases [`ubuntu-latest`, `ubuntu-22.04`, `ubuntu-20.04`, `linux-musl`];
- Update `crunch-update.sh` script to support multiple releases;

## [0.19.0] - 2025-03-06

## New
- [#55] Add nomination pools claim commission feature by adding option `--enable-pool-claim-commission` or env variable `CRUNCH_POOL_CLAIM_COMMISSION_ENABLED=true`. Note that the nomination pool root account has to explicitly sets this feature via extrinsic `set_commission_claim_permission`;

## Changed
- [#56] Crunch binary release has been changed to `ubuntu-latest`;
- Update subxt to `v0.40.0`
- Update metadata polkadot/1004001
- Update metadata people-polkadot/1004000
- Update metadata kusama/1004001
- Update metadata people-kusama/1004000
- Update metadata paseo/1003004
- Update metadata people-paseo/1003003
- Update metadata westend/1018000
- Update metadata people-westend/1018000

## [0.18.1] - 2024-09-17

## Changed
- Fix People-Paseo chain-spec

## [0.18.0] - 2024-09-17

## Changed
- Update metadata polkadot/1003000
- Update metadata paseo/1003000
- Add metadata people-paseo/1002007

## [0.17.1] - 2024-09-5

## Changed
- Update metadata kusama/1003000

## [0.17.0] - 2024-07-25

## New
- Fetch identities from people-polkadot chain with use of `--substrate-people-ws-url`

## Changed
- Update metadata polkadot/1002007
- Update metadata people-polkadot/1002006

## [0.16.0] - 2024-06-05

## New
- Fetch identities from people-kusama/people-westend chain with use of `--substrate-people-ws-url` (Only available for Kusama & Westend)
- Allow crunch to run payouts and exit with run mode `once`; useful if you would like to setup crunch as cronjob;
- Connect to a specific network via smoldot with flag `--enable-light-client`; no need to specify an RPC endpoint via `--substrate-ws-url`; usage example `crunch kusama --enable-light-client rewards once`.
- Group payouts and report messages by validators main identity using flag `--enable-group-identity`; useful to organize long list of validator stashes with different identities; usage example `crunch kusama --enable-group-identity rewards era`.

## Changed
- Update subxt v0.37.0 (make use of unstable subxt RPC reconnection and unstable light client features)
- Iterate transaction progress to only evaluate events when block is finalized, log and drop all other states
- Generate only metadata from specific pallets
- Update metadata polkadot/1002000
- Update metadata kusama/1002001
- Update metadata westend/1011000
- Update metadata paseo/1002000

## [0.15.0] - 2024-04-26

## New
- Add tip for block author optional with `--tx-tip`
- Add transaction mortal optional with `--tx-mortal-period`
- Add github personal access token with `--github-pat` to grant access to a list of stashes defined in a private github repo
- Sort stashes by identity, no-identity and push warnings to bottom

## Changed
- Review no bonded controller message
- Fix oversubscribed validators by checking all unclaimed pages

## [0.14.0] - 2024-04-21

## Changed
- Fixes breaking changes from latest Polkadot v1.2.0 runtime upgrade
- Update metadata Polkadot runtime/1002000

## [0.13.2] - 2024-04-19

## Changed
- Fixes claimed rewards from new storage 'staking.claimed_rewards'

## [0.13.1] - 2024-04-19

## Changed
- Fixes unclaimed eras from new storage 'staking.eras_stakers_paged'
- update base64 crate version

## [0.13.0] - 2024-04-18

## Changed
- Fixes breaking changes from latest Kusama v1.2.0 runtime upgrade
- Update metadata Kusama runtime/1002000

## [0.12.2] - 2024-02-29

## Bugfix
- Fixes issue [#39](https://github.com/turboflakes/crunch/issues/39), removes control characters from seed file, before parsing content.

## [0.12.1] - 2024-02-29

## Bugfix
- Since v0.11.0 only mnemonic phrases were parsed correctly from the seed file, this release fixes this issue, allowing secret keys, mnemonic phrases or uri to be interpreted correctly.

## [0.12.0] - 2024-02-28

## New
- Add `medium` flag as another verbosity option
- Add support for Paseo Testnet
- Add metadata Paseo runtime/1001002

## Changed
- Update metadata Polkadot runtime/1001002
- Update metadata Kusama runtime/1001002

## [0.11.2] - 2024-01-26

## Changed
- Fix unsecure Urls.

## [0.11.1] - 2024-01-26

## Changed
- Fix `onet_api_url` to depend on the connected chain and remove default endpoint.
- Show nomination pool threshold value in the report
- Update subxt v0.34.0
- Update metadata Polkadot runtime/1000000
- Update metadata Kusama runtime/1001000
- Disable Westend runtime/1006000

## [0.10.1] - 2023-07-14

## New
- introducing option `--enable-pool-only-operator-compound` to allow for permissionless compound rewards of pool operators only
- introducing flag `--enable-pool-compound-threshold` to allow a threshold to be set. Only rewads higher than the threshold are triggered for compound.

## change
- NOTE: option and respective flags have been renamed:
 `--enable-all-nominees-payouts` -> `--enable-pool-all-nominees-payout`
 `--enable-active-nominees-payout` -> `--enable-pool-active-nominees-payout`
 `CRUNCH_ALL_NOMINEES_PAYOUTS_ENABLED` -> `CRUNCH_POOL_ALL_NOMINEES_PAYOUT_ENABLED`
 `CRUNCH_ACTIVE_NOMINEES_PAYOUT_ENABLED` -> `CRUNCH_POOL_ACTIVE_NOMINEES_PAYOUT_ENABLED`

## [0.10.0] - 2023-07-11

## change
- batch pool members with permissionless compound rewards defined
- fetch ONE-T grades

## Changed
- Update subxt v0.29.0
- use `force_batch`
- change `error_interval` to base pow function
- Support only Westend, Kusama, Polkadot (if nedeed other substrate-based chains could easily clone and adapt required changes)
- Update metadata Kusama runtime/9430

## [0.9.6] - 2023-06-15

- Update metadata Polkadot runtime/9420
- Update metadata Westend runtime/9430

## [0.9.5] - 2023-05-25

- Update metadata Kusama runtime/9420
- Update metadata Westend runtime/9420

## [0.9.3] - 2023-02-15

- Fixes active nominees stashes from previous era and not current from version 0.9.2

## [0.9.2] - 2023-02-15

- Fixes active nominees stashes from version 0.9.1

## [0.9.1] - 2023-02-15

### New
- Add optional flag 'enable-all-nominees-payouts'. Since this version, by default 'crunch' will only trigger payouts for active nominees that the Pool stake allocation was active in the previous era. The presence of this optional flag makes 'crunch' to try and trigger payouts to all nominees regardless if they were active or not.

## [0.9.0] - 2023-02-15

### New
- Add optional flag 'enable-unique-stashes'. From all given stashes `crunch` will sort by stash address and remove duplicates.
- Add optional flag 'pool-ids' or environement variable 'CRUNCH_POOL_IDS'. `crunch` will try to fetch the nominees of the respective pool id predefined here before triggering the respective payouts.

### Changed
- Update metadata Polkadot runtime/9360
- Update metadata Kusama runtime/9370
- Update metadata Westend runtime/9380

## [0.8.3] - 2023-01-20

### Changed
- Remove leading and trailing whitespace from remote stashes file
- Update metadata Polkadot runtime/9340
- Update metadata Kusama runtime/9360
- Update metadata Westend runtime/9370

## [0.8.1] - 2022-12-19

### Changed
- Aleph main & test networks [PR 23](https://github.com/turboflakes/crunch/pull/23)

## [0.8.0] - 2022-12-15

### Changed
- subxt v0.25.0
- Update metadata Kusama runtime/9350
- Update metadata Westend runtime/9350

## [0.7.1] - 2022-12-15

- Update metadata Kusama runtime/9350
- Update metadata Westend runtime/9350

## [0.6.3] - 2022-12-15

### Changed

- Update metadata Kusama runtime/9320
- Update metadata Westend runtime/9330

## [0.6.2] - 2022-12-06

### Changed

- Update metadata Kusama runtime/9320
- Update metadata Westend runtime/9330

## [0.6.1] - 2022-11-11

### Changed

- Update metadata Polkadot runtime/9300
- Update metadata Westend runtime/9320
- Aleph main & test networks [PR 20](https://github.com/turboflakes/crunch/pull/20)

## [0.6.0] - 2022-11-01

### New
- Add optional environement variable 'CRUNCH_EXISTENTIAL_DEPOSIT_FACTOR_WARNING' so that the factor value could be configurable per chain. Default value is 2. The recommended values based on the existential deposits is factor 2x for Polkadot and 1000x for Kusama.

### Changed
- support `subxt` [v0.24.0](https://github.com/paritytech/subxt/releases/tag/v0.24.0)
- Update metadata Westend runtime/9310

## [0.5.15] - 2022-10-26

### Changed

- Update metadata Kusama runtime/9300
- Update metadata Westend runtime/9300

## [0.5.14] - 2022-10-18

### Changed

- Update metadata Polkadot runtime/9291

## [0.5.13] - 2022-09-28

### Changed

- Update `subxt v0.22.0`
- Update metadata Kusama runtime/9291
- Update metadata Westend runtime/9290

## [0.5.11] - 2022-09-07

### New

- Add support for Tidechain's testnet Lagoon [PR15](https://github.com/turboflakes/crunch/pull/15)

### Changed

- Update metadata Polkadot runtime/9280

## [0.5.10] - 2022-09-07

### Changed

- Update metadata Kusama runtime/9280

## [0.5.9] - 2022-09-07

### Changed
- Change Kusama low balance warning to 1000 x ed
- Update metadata Polkadot runtime/9270

## [0.5.8] - 2022-08-31

- Update metadata Kusama runtime/9271
- Update metadata Kusama runtime/9280
- Update metadata Aleph Zero Testnet runtime/30 [PR 14](https://github.com/turboflakes/crunch/pull/14)

## [0.5.7] - 2022-08-09

- Add support for Aleph Zero Mainnet
- Add metadata Aleph Zero Mainnet runtime/12
- Update metadata Polkadot runtime/9260
- Update metadata Westend runtime/9271

## [0.5.6] - 2022-08-06

### Added
- Add support for Aleph Zero Testnet
- Add metadata Aleph Zero Testnet runtime/30

## [0.5.5] - 2022-07-26

- Reduce number of recursive attempts to only once
- Update metadata Polkadot runtime/9250
- Update metadata Kusama runtime/9260

## [0.5.4] - 2022-07-23

- Fix enable view and subscription modes - these modes were wrongly disabled in the previous released

## [0.5.3] - 2022-07-21

- Fix recursive call in case of batch interrupted
- Update `subxt v.0.21.0`
- Update metadata Polkadot runtime/9230
- Update metadata Kusama runtime/9250
- Update metadata Westend runtime/9260

## [0.5.2] - 2022-03-17

### Changed

- Fix skipping finalised blocks by updating `subxt` crate dependency to latest commit `8b19549` - version 0.18.1
- Review summary description, with the addition of the number of stashes that had the previous era claimed earlier.
- Update metadata Polkadot runtime/9170
- Update metadata Westend runtime/9170

## [0.5.1] - 2022-02-28

- Fix summary with a clickable details on top for with `is-short` flag.
- Update metadata Kusama runtime/9170

## [0.5.0] - 2022-02-22

### Added

- Add summary with a clickable details on top.
- Add optional flag 'stashes-url' so that a list of stashes could be fetched from a remote endpoint

### Changed

- After the end of an era the payout is triggered after a random waiting period (up to a maximum of 120 seconds). This aims to prevent a block race from all `crunch` bots at the beginning of each era.
- Fix `Already claimed` rewards issue
- Fix parity default endpoints by defining port number
- Update metadata Polkadot runtime/9151
- Update metadata Kusama runtime/9160
- Update metadata Westend runtime/9160

## [0.4.1] - 2022-01-14

### Changed

- Update metadata Westend runtime/9150
- Update metadata Kusama runtime/9150

## [0.4.0] - 2022-01-11

### Changed

- Changed single payouts for batch calls
- Update `subxt` dependency to revision `41bd8cc`

### Added

- Add `maximum-calls` flag with default value of 8. By default `crunch` collects all the outstanding payouts from previous eras and group all the extrinsic payout calls in group of 8 (or whatever value defined by this flag) so that a single batch call per group can be made. Using batch calls rather than single payouts we could expect a significant drop in transaction fees and a significat increase on `crunch` performance.

### Changed

## [0.3.2] - 2021-11-03

### Changed

- Update substrate-subxt dependency. Subscription to `EraPaid` event should now run as expected without panic events every session.
- Fix loading `CRUNCH_SUBSTRATE_WS_URL` environment from `.env` file
- Default `error-interval` time reduced to 5 min
- Note: Batch calls are still not supported on this version

## [0.3.0] - 2021-10-17

### Changed

- Fix substrate-subxt dependency with support for metadata v14
- Note: Batch calls are not supported on this version -> potentially on next release

## [0.2.2] - 2021-10-09

### Added

- Add `maximum_history_eras` flag with default value of 4. Note: This flag is only valid if `short` flag is also present. By default `crunch` will only check for unclaimed rewards in the last 4 eras rather than the last 84 as in previous versions. If running `crunch` in verbose mode the check in the last 84 eras still apply by default, since we would like to keep showing information regarding Inclusion and total Crunched eras for all history.

### Changed

- Fix loading configuration variables specified in `.env` file.
- Fix bug for new chains that have `current_era` value lower than `history_depth` constant.

## [0.2.1] - 2021-09-30

### Added

- Add bash script `crunch-update.sh` for easier install or crunch update

### Changed

- Identify an excellence performance by using Interquartile Range(IQR)
- Update substrate-subxt dependency (`Substrate_subxt error: Scale codec error: Could not decode 'Outcome', variant doesn't exist` error fixed)

## [0.2.0] - 2021-09-25

### Added

- Support a batch of dispatch calls by default
- Additional 99.9% confidence interval for performance reaction
- Additional randomness on emojis and flakes messages

### Changed

- Fix typos
- Improve identity
- Notification message refactored
- Minor messages typo changes
- Update substrate-subxt dependency
- Multilingual hello message

## [0.1.18] - 2021-09-15

### Added

- Optional flag --error-interval to adjust the time between crunch automatic restart in case of error
- Additional mode 'era' that subscribes to EraPaid on-chain event to trigger the payout

### Changed

- use 99% confidence interval for performance reaction
- update substrate-subxt dependency
- fix optional flag --debug

## [0.1.17] - 2021-09-07

### Added

- Warn if signer account free funds are lower than 2x the Existential Deposit
- Link validator identity to subscan.io
- Always show points and total reward amount plus good performance reaction

### Changed

- Remove *nothing to crunch this time message* if short flag is present
- Fix substrate-subxt dependency by commit hash
- Fix changelog - latest version comes first
- Change finalize block link to subscan.io

## [0.1.15] - 2021-09-03

### Added

- Optional flag --short to display only essencial information

### Changed

- Small adjustments on overal notifications

## [0.1.14] - 2021-08-30

### Changed

- Fix event 'Rewarded' active on chains runnimg Runtime 9090

## [0.1.13] - 2021-08-19

### Changed

- Update dependencies

## [0.1.12] - 2021-08-13

### Added

- Show validator era points and average

## [0.1.11] - 2021-08-13

### Changedd

- Improve message readability
- Only send one matrix message per run

## [0.1.9] - 2021-08-07

### Added

- Add changelog (this file)
- Check if stash is currently in active set
- Improve messages readability

### Changed

- Highlight validator name in logs
- By default connect to local substrate node if no chain is specified

## [0.1.8] - 2021-08-05

### Added

- First release
- Claim staking rewards for one or a list of Validators
- Only inspect for claimed or unclaimed eras
- Easily connect to westend, kusama or polkadot Parity public nodes
- Set optional matrix bot
- Set `flakes` as default subcommand and optional `rewards` for a more regular logging/messages
