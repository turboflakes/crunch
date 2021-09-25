# crunch &middot; ![latest release](https://github.com/turboflakes/crunch/actions/workflows/create_release.yml/badge.svg)

<p align="center">
  <img src="https://github.com/turboflakes/crunch/blob/assets/crunchbot-github-header.png?raw=true">
</p>

`crunch` is a command-line interface (CLI) to claim staking rewards every X hours for Substrate-based chains.

## Why use `crunch`

To automate payout of staking rewards for just one or a list of Validators every X hours.

To get notified about the amount and rate of the total staking rewards each Validator and their Nominators got

To be informed about era stats for each Validator, e.g. inclusion rate, claimed rewards rate, era points trend, active for current era

To easily inspect about any unclaimed eras for a given Validator stash

To promote Validators by publicly publish their automated staking rewards to a public **Crunch Bot** room

For Nominators in private or public rooms check their chosen Validators rewards performance

## Installation

```bash
#!/bin/bash
# create `crunch-bot` directory
$ mkdir ~/crunch-bot
# download `crunch` latest version
$ wget -P ~/crunch-bot https://github.com/turboflakes/crunch/releases/download/v0.2.0/crunch
# make `crunch` binary file executable
chmod +x ~/crunch-bot/crunch
```

## Configuration

Create a configuration file `.env` inside `crunch-bot` folder and copy the default variables from [`.env.example`](https://github.com/turboflakes/crunch/blob/main/.env.example) (Note: `.env` is the default name and a hidden file, if you want something different you can adjust it later with the option `crunch --config-path ~/crunch-bot/config_kusama.env` )

```bash
#!/bin/bash
# create/open a file with a file editor (Vim in this case) and add/change the configuration
# variables with your own personal values
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

### Crunch Bot ([Matrix](https://matrix.org/))

If you set up `crunch` on your server with a matrix user 👉  you get your own **Crunch Bot**.

To enable **Crunch Bot** you will need to create a specific account on Element or similar and  copy the values to the respective environment variables `CRUNCH_MATRIX_BOT_USER` and `CRUNCH_MATRIX_BOT_PASSWORD` like in the configuration example file [`.env.example`](https://github.com/turboflakes/crunch/blob/main/.env.example). You may also want to set your regular matrix user to the environment variable `CRUNCH_MATRIX_USER`. So that **Crunch Bot** could create a private room and send in messages. By default **Crunch Bot** will automatically invite your regular matrix user to a private room. Also by default **Crunch Bot** will send a copy of the messages to the respective network public room for which is connected to.

### Public Rooms available

Join and read the messages history of all the Public Rooms for which **Crunch Bots** are sending messages:

<table style="width:100%;" cellspacing="0" cellpadding="0">
  <tr>
    <td style="width: 100px;">
        <img style="width: 80px;" src="https://github.com/turboflakes/crunch/blob/assets/crunchbot-westend-room-128.png?raw=true">
    </td>
    <td><a href="https://matrix.to/#/%23westend-crunch-bot:matrix.org" target="_blank">Westend Crunch Bot (Public)</a></td>
  </tr>
  <tr>
    <td style="width: 100px;">
        <img style="width: 80px;" src="https://github.com/turboflakes/crunch/blob/assets/crunchbot-kusama-room-128.png?raw=true">
    </td>
    <td><a href="https://matrix.to/#/%23kusama-crunch-bot:matrix.org" target="_blank">Kusama Crunch Bot (Public)</a></td>
  </tr>
  <tr>
    <td style="width: 100px;">
        <img style="width: 80px;" src="https://github.com/turboflakes/crunch/blob/assets/crunchbot-polkadot-room-128.png?raw=true">
    </td>
    <td><a href="https://matrix.to/#/%23polkadot-crunch-bot:matrix.org" target="_blank">Polkadot Crunch Bot (Public)</a></td>
  </tr>
</table>

### Crunch Bot messages

![crunch bot notification messages example](https://github.com/turboflakes/crunch/blob/assets/westend-crunch-bot.png?raw=true)

## Usage

If you have been doing `crunch` configuration as described in previous steps (assuming `.env` and `.private.seed` defined inside `~/crunch-bot` folder), run `crunch` when `~/crunch-bot` folder is your current working directory. Otherwise you will have to specify `.env` and `.private.seed` custom paths.

```bash
#!/bin/bash
# set ~/crunch-bot your current working directory
$ cd ~/crunch-bot
```

By default `crunch` tries to connect to your local substrate node on the default websocket port `ws://127.0.0.1:9944`. This can be changed by typing one of polkadot main chains - westend, kusama or polkadot. Or by changing the substrate websocket url with the option `--substrate-ws-url`.

`crunch` default sub command is `flakes`, there are fun messages if you stick with it, or you can choose the regular sub command `rewards` rather than `flakes`. As you prefer. Both sub commands are identical in terms of job execution. But logs, messages/notifications differ.

Essentially `crunch` motto is enjoy **Crunch Bot** while `crunch flakes` :)

If all has been set as previously described `crunch` should be ready with just the following options:

```bash
#!/bin/bash
# and than simple run crunch with default options to connect to your local node
$ crunch flakes
# or specify directly which network crunch will try to connect
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
```

If you need more customization run help to check all sub commands, flags and options.

Note: All flags and options are also available through environment variables if defined in `.env` configuration file. You can choose which way you want to configure `crunch`. Take in consideration that if the same variable is defined on both sides e.g. defined in `.env` and through CLI flag/option, `crunch` will take the value defined by CLI.

```bash
#!/bin/bash
# if you need a custom crunch check all the options and flags available
$ crunch help
```

```bash
#!/bin/bash
# or help for any subcommand like
$ crunch rewards --help

USAGE:
    crunch rewards [FLAGS] [OPTIONS] [MODE]

FLAGS:
        --debug                              Prints debug information verbosely.
        --disable-matrix                     Disable matrix bot for 'crunch rewards'. (e.g. with this flag active
                                             'crunch rewards' will not send messages/notifications about claimed or
                                             unclaimed staking rewards to your private or public 'Crunch Bot' rooms)
                                             (https://matrix.org/)
        --disable-matrix-bot-display-name    Disable matrix bot display name update for 'crunch rewards'. (e.g. with
                                             this flag active 'crunch rewards' will not change the matrix bot user
                                             display name)
        --disable-public-matrix-room         Disable notifications to matrix public rooms for 'crunch rewards'. (e.g.
                                             with this flag active 'crunch rewards' will not send messages/notifications
                                             about claimed or unclaimed staking rewards to any public 'Crunch Bot' room)
    -h, --help                               Prints help information
        --short                              Display only essential information (e.g. with this flag active 'crunch
                                             rewards' will only send essential messages/notifications about claimed
                                             rewards)
    -V, --version                            Prints version information

OPTIONS:
        --error-interval <error-interval>
            Interval value (in minutes) from which 'crunch' will restart again in case of a critical error. [default:
            30]
        --matrix-bot-password <matrix-bot-password>    Password for the 'Crunch Bot' matrix user sign in.
        --matrix-bot-user <matrix-bot-user>
            Your new 'Crunch Bot' matrix user. e.g. '@your-own-crunch-bot-account:matrix.org' this user account will be
            your 'Crunch Bot' which will be responsible to send messages/notifications to your private or public 'Crunch
            Bot' rooms.
        --matrix-user <matrix-user>
            Your regular matrix user. e.g. '@your-regular-matrix-account:matrix.org' this user account will receive
            notifications from your other 'Crunch Bot' matrix account.
    -m, --maximum-payouts <maximum-payouts>
            Maximum number of unclaimed eras for which an extrinsic payout will be submitted. (e.g. a value of 4 means
            that if there are unclaimed eras in the last 84 the maximum unclaimed payout calls for each stash address
            will be 4). [default: 4]
    -f, --seed-path <FILE>
            Sets a custom seed file path. The seed file contains the private seed phrase to Sign the extrinsic payout
            call. [default: .private.seed]

ARGS:
    <MODE>    Sets how often staking rewards should be claimed from unclaimed eras. (e.g. the option 'era' sets
              'crunch' task to run as soon as the EraPaid on-chain event is triggered; the option 'daily' sets
              'crunch' task to be repeated every 24 hours; option 'turbo' sets 'crunch' task to be repeated every 6
              hours) [default: era]  [possible values: era, daily, turbo]
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

Note: You can run `crunch` inside a tmux session and leave it, or using something like `systemd` to run `crunch` on server restarts for example. By default `crunch` will wake up every X hours to claim rewards if there are any to claim.

## Development / Build from Source

If you'd like to build from source, first install Rust.

```bash
curl https://sh.rustup.rs -sSf | sh
```

If Rust is already installed run

```bash
rustup update
```

Verify Rust installation by running

```bash
rustc --version
```

Once done, finish installing the support software

```bash
sudo apt install build-essential git clang libclang-dev pkg-config libssl-dev
```

Build `crunch` by cloning this repository

```bash
#!/bin/bash
$ git clone http://github.com/turboflakes/crunch
```

Compile `crunch` package with Cargo

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
- <a href="https://dribbble.com/shots/6453317-Mac-And-Cross-Bones" target="_blank">Jetpacks and Rollerskates</a> - Illustration work heavily inspired **Crunch Bot** logo.

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

### Quote

> "Study hard what interests you the most in the most undisciplined, irreverent and original manner possible."
― Richard Feynmann

__

Enjoy `crunch`
