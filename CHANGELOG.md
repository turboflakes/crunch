# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Identify an excellence performance using Interquartile Range(IQR)

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
