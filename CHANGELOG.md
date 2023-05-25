# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
