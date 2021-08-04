# crunch &middot; ![latest release](https://github.com/turboflakes/crunch/actions/workflows/create_release.yml/badge.svg)

<p align="center">
  <img src="https://github.com/turboflakes/crunch/blob/assets/crunchbot-github-header.png?raw=true">
</p>

`crunch` is a command-line interface (CLI) to claim staking rewards every X hours for Substrate-based chains.

## Why use `crunch`

To automate payout of staking rewards for just one or list of Validators every X hours.

To get notified about the amount and fraction of the Total Rewards each Validator and their Nominators received.

To easily inspect about any unclaimed eras.

To promote Validator/s by publicly publish their automated staking rewards

## Installation

```bash
#!/bin/bash
# create `crunch-bot` directory
$ mkdir ~/crunch-bot
# download `crunch` latest version
$ wget -P ~/crunch-bot https://github.com/turboflakes/crunch/releases/download/v0.1.7/crunch
# make `crunch` binary file executable
chmod +x ~/crunch-bot/crunch
```

## Configuration

Create a configuration file `.env` inside `crunch-bot` folder and copy the default variables from [`.env.example`](https://github.com/turboflakes/crunch/blob/main/.env.example) (Note: `.env` is the default name and a hidden file, if you want something different you can adjust it later with the option `crunch --config-path ~/crunch-bot/config_kusama.env` )

```bash
#!/bin/bash
# create/open a file with a file editor (Vim in this case) and add/change the configuration variables with your own personal values
$ vi ~/crunch-bot/.env
# when ready write and quit (:wq!)
```

Configuration file example: [`.env.example`](https://github.com/turboflakes/crunch/blob/main/.env.example)

```bash
# crunch CLI configuration variables 
#
# [CRUNCH_STASHES] Validator stash addresses for which 'crunch flakes', 'crunch rewards'
# or 'crunch view' will be applied. 
# If needed specify more than one (e.g. stash_1,stash_2,stash_3).
CRUNCH_STASHES=5GTD7ZeD823BjpmZBCSzBQp7cvHR1Gunq7oDkurZr9zUev2n
#
# [CRUNCH_SUBSTRATE_WS_URL] Substrate websocket endpoint for which 'crunch' will try to
# connect. (e.g. wss://kusama-rpc.polkadot.io) (NOTE: substrate_ws_url takes precedence
# than <CHAIN> argument) 
#CRUNCH_SUBSTRATE_WS_URL=wss://westend-rpc.polkadot.io
#
# [CRUNCH_MAXIMUM_PAYOUTS] Maximum number of unclaimed eras for which an extrinsic payout
# will be submitted. (e.g. a value of 4 means that if there are unclaimed eras in the last
# 84 the maximum unclaimed payout calls for each stash address will be 4). [default: 4]
CRUNCH_MAXIMUM_PAYOUTS=4
#
# [CRUNCH_SEED_PATH] File path containing the private seed phrase to Sign the extrinsic 
# payout call. [default: .private.seed]
#CRUNCH_SEED_PATH=.private.seed.example
#
# Crunch Bot (matrix) configuration variables
CRUNCH_MATRIX_USER=@your-regular-matrix-account:matrix.org
CRUNCH_MATRIX_BOT_USER=@your-own-crunch-bot-account:matrix.org
CRUNCH_MATRIX_BOT_PASSWORD=anotthateasypassword
```

Create a seed private file `.private.seed` inside `crunch-bot` folder and write the private seed phrase of the account responsible to sign the extrinsic payout call as in [`.private.seed.example`](https://github.com/turboflakes/crunch/blob/main/.private.seed.example) (Note: `.private.seed` is the default name and a hidden file, if you want something different you can adjust it later with the option `crunch flakes --seed-path ~/crunch-bot/.kusama.private.seed` )

```bash
#!/bin/bash
# create a file with a file editor (Vim in this case) and write the private seed phrase 
# of the account responsible to sign the extrinsic payout call
$ vi ~/crunch-bot/.private.seed
# when ready write and quit (:wq!)
```

### CRUNCH BOT ([Matrix](https://matrix.org/))

If you set up `crunch` on your server with a matrix user 👉  you get your own CRUNCH BOT.

To enable CRUNCH BOT you will need to create a specific account on Element or similar and  copy the values to the respective environment variables `CRUNCH_MATRIX_BOT_USER` and `CRUNCH_MATRIX_BOT_PASSWORD` like in the configuration example file [`.env.example`](https://github.com/turboflakes/crunch/blob/main/.env.example). You may also want to set your regular matrix user to the environment variable `CRUNCH_MATRIX_USER`. So that `crunch` bot could create a private room and send in messages. By default `crunch` bot will automatically invite your regular matrix user to a private room and send the same messages to a public room specific to the network which is connected to.

### Public Rooms available

You can join the crew now and read the messages history of all the **CRUNCH BOTS** that send messages to the following Public Rooms:

<table style="width:100%;" cellspacing="0" cellpadding="0">
  <tr>
    <td>
        <img src="https://github.com/turboflakes/crunch/blob/assets/crunchbot-westend-room-128.png?raw=true">
    </td>
    <td><a href="https://matrix.to/#/%23westend-crunch-bot:matrix.org" target="_blank">Westend Crunch Bot (Public)</a></td>
  </tr>
  <tr>
    <td>
        <img src="https://github.com/turboflakes/crunch/blob/assets/crunchbot-kusama-room-128.png?raw=true">
    </td>
    <td><a href="https://matrix.to/#/%23kusama-crunch-bot:matrix.org" target="_blank">Kusama Crunch Bot (Public)</a></td>
  </tr>
  <tr>
    <td>
        <img src="https://github.com/turboflakes/crunch/blob/assets/crunchbot-polkadot-room-128.png?raw=true">
    </td>
    <td><a href="https://matrix.to/#/%23polkadot-crunch-bot:matrix.org" target="_blank">Polkadot Crunch Bot (Public)</a></td>
  </tr>
</table>

### Crunch Bot Messages

![crunch bot notification messages example](https://github.com/turboflakes/crunch/blob/assets/westend-crunch-bot.png?raw=true)

## Usage

If you have been doing `crunch` configuration as described in previous steps (assuming `.env` and `.private.seed` defined inside `~/crunch-bot` folder), by default execute `crunch` when `crunch-bot` folder is your current working directory. Otherwise you will have to specify `.env` and `.private.seed` custom paths.

```bash
#!/bin/bash
# set ~/crunch-bot your current working directory
$ cd ~/crunch-bot
```

### Moto

Enjoy **CRUNCH BOT** while `crunch flakes` :)

If you prefer you can choose the sub command `rewards` rather than `flakes` or vice versa. As you prefer. Both sub commands are identical in terms of job execution. But logs, messages/notifications differ.

```bash
#!/bin/bash
# and than simple run crunch with default options for Westend network
$ crunch westend flakes
# or for Kusama network and claiming rewards every 6 hours
$ crunch kusama flakes turbo
# or for Polkadot network and claiming rewards once a day
$ crunch polkadot flakes daily
# or run crunch a bit more boring for Westend network
$ crunch westend rewards turbo
# or for Kusama network and claiming rewards once a day
$ crunch kusama rewards daily
# or for Polkadot network and claiming rewards once a day
$ crunch polkadot rewards daily
# if you need a custom crunch check all the options and flags available
$ crunch help
# or help for any subcommand like
$ crunch flakes --help
```

If you need more customization run help to check all sub commands, flags and options.

Note: All flags and options are also available through environment variables if defined in `.env` configuration file. You can choose which way you want to configure `crunch`. But take in consideration that if the same variable is defined on both sides e.g. defined in `.env` and through CLI flag/option, the value `crunch` will take in consideration will be de one defined by CLI.

```bash
#!/bin/bash
# if you need a custom crunch check all the options and flags available
$ crunch help
# or help for any subcommand like
$ crunch flakes --help
```

Also if you just want to know for the stash accounts defined in the confguration file (`.env`), which eras from the last 84 have already been claimed or unclaimed, you can simply run `crunch view`

Note: This option only logs information on the terminal

```bash
#!/bin/bash
# run crunch for Westend network and claiming rewards every 6 hours
$ crunch westend view
# or for Kusama network
$ crunch kusama view
# or for Polkadot network
$ crunch polkadot view
```

Note: You can run `crunch` inside a tmux session and leave it, or using something like `systemd` to auto start on server restarts for example. By default crunch will wake up every X hours to claim rewards if any to claim.

## Development

Clone the repository and compile the package with Cargo

```bash
#!/bin/bash
$ git clone http://github.com/turboflakes/crunch
```

Compile the crunch binary

```bash
#!/bin/bash
$ cargo build
```

And then run it

```bash
#!/bin/bash
$ ./target/debug/crunch westend flakes daily
```

Otherwise, recompile the code on changes and run the binary

```bash
#!/bin/bash
$ cargo watch -x 'run --bin crunch'
```

## Inspiration

Similar projects that had influence in crunch design.

- <a href="https://github.com/canontech/staking-payouts" target="_blank">staking-payouts</a> - CLI to make staking payout transactions for Substrate FRAME-based chains.
- <a href="https://github.com/stakelink/substrate-payctl" target="_blank">substrate-payctl</a> - Simple command line application to control the payouts of Substrate validators (Polkadot and Kusama among others).
- <a href="https://github.com/stakelink/substrate-payctl" target="_blank">Jetpacks and Rollerskates</a> - Illustration work heavily inspired **CRUNCH BOT** logo.

## Collaboration

Have an idea for a new feature, a fix or you found a bug, please open an [issue](https://github.com/turboflakes/crunch/issues) or submit a [pull request](https://github.com/turboflakes/crunch/pulls).

Any feedback is welcome.

## About

`crunch` was made by <a href="https://turboflakes.com" target="_blank">TurboFlakes</a>.

TurboFlakes is also an independent validator in the Kusama Network and eligible in the Kusama's Thousand Validators Programme, aka <a href="https://thousand-validators.kusama.network/#/leaderboard/FZsMKYHoQG1dAVhXBMyC7aYFYpASoBrrMYsAn1gJJUAueZX" target="_blank" rel="noreferrer">1KV</a>.

If you like this project ✌️ Share our work and support us with your nomination or tip ✨💙

- **Polkadot**: 12gPFmRqnsDhc9C5DuXyXBFA23io5fSGtKTSAimQtAWgueD2
- **Kusama**: FZsMKYHoQG1dAVhXBMyC7aYFYpASoBrrMYsAn1gJJUAueZX

You could also Star the Github project :)

### License

`crunch` is [MIT licensed](./LICENSE).

### Finish

> "Study hard what interests you the most in the most undisciplined, irreverent and original manner possible."
― Richard Feynmann

Enjoy `crunch`
