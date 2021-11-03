# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
