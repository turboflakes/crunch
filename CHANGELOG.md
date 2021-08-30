# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.8] - 2021-08-05

### Added

- First release
- Claim staking rewards for one or a list of Validators
- Only inspect for claimed or unclaimed eras
- Easily connect to westend, kusama or polkadot Parity public nodes
- Set optional matrix bot
- Set `flakes` as default subcommand and optional `rewards` for a more regular logging/messages

## [0.1.9] - 2021-08-07

### Added

- Add changelog (this file)
- Check if stash is currently in active set
- Improve messages readability

### Changed

- Highlight validator name in logs
- By default connect to local substrate node if no chain is specified

## [0.1.11] - 2021-08-13

### Changed

- Improve message readability
- Only send one matrix message per run

## [0.1.12] - 2021-08-13

### Add

- Show validator era points and average

## [0.1.13] - 2021-08-19

### Change

- Update dependencies

## [0.1.14] - 2021-08-30

### Change

- fix rewarded event dependency
