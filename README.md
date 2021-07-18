# crunch

Crunch is a command-line interface (CLI) to claim staking rewards (flakes) every X hours for Substrate-based chains

![latest release](https://github.com/turboflakes/yummies/actions/workflows/create_release.yml/badge.svg)

## Run

```bash
#!/bin/bash
$ cargo run
```

## Development

Recompile the code on changes and run the binary

```bash
#!/bin/bash
$ cargo watch -x 'run --bin crunch'
```

### Inspiration

Similar projects that had influence in crunch design.

- [CLI to make staking payout transactions for Substrate FRAME-based chains.](https://github.com/canontech/staking-payouts)
- [Simple command line application to control the payouts of Substrate validators (Polkadot and Kusama among others).](https://github.com/stakelink/substrate-payctl)
