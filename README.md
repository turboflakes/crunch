# crunch

Crunch is a command-line interface (CLI) to claim staking rewards (flakes) every X hours for Substrate-based chains.

If you leave `crunch` set up and running on your server 👉 you get your own **Crunch Bot**

![latest release](https://github.com/turboflakes/crunch/actions/workflows/create_release.yml/badge.svg)

## Install and run Crunch Bot

Create a `crunch-bot` directory
```bash
#!/bin/bash
$ mkdir ~/crunch-bot
```

Download `crunch` latest version 
```bash
#!/bin/bash
$ wget -P ~/crunch-bot https://github.com/turboflakes/crunch/releases/download/v0.1.7/crunch
```

Make `crunch` binary file executable
```bash
#!/bin/bash
$ chmod +x ~/crunch-bot/crunch
```

Create a configuration file `.env` inside `crunch-bot` folder and copy the default variables from [`.env.example`](https://github.com/turboflakes/crunch/blob/main/.env.example) (Note: `.env` is the default name and hidden file, if you want something different you can adjust it with the option `--config-path ~/crunch-bot/config_kusama.env` )

```bash
#!/bin/bash
# create configuration file .env inside crunch-bot folder
$ touch ~/crunch-bot/.env
# open file and edit configuration variables to adjust your personal values
$ vi ~/crunch-bot/.env
# once in vim change configuration variables, and than write and quit
$ :wq!
```

Configuration file example: [`.env.example`](https://github.com/turboflakes/crunch/blob/main/.env.example)

```bash
# crunch CLI configuration variables 
#
# [CRUNCH_STASHES] Validator stash addresses for which 'crunch flakes', 'crunch rewards' or 'crunch view' will be applied. 
# If needed specify more than one (e.g. stash_1,stash_2,stash_3).
CRUNCH_STASHES=5E7QrLDGHVU3uP4RjnKsryecZXcWQ7oJwWFypRAoHCL1nAnG
#
# [CRUNCH_SUBSTRATE_WS_URL] Substrate websocket endpoint for which 'crunch' will try to connect. 
# (e.g. wss://kusama-rpc.polkadot.io) (NOTE: substrate_ws_url takes precedence than <CHAIN> argument)
#CRUNCH_SUBSTRATE_WS_URL=wss://westend-rpc.polkadot.io
#
# [CRUNCH_MAXIMUM_PAYOUTS] Maximum number of unclaimed eras for which an extrinsic payout will be submitted. 
# (e.g. a value of 4 means that if there are unclaimed eras in the last 84 the maximum unclaimed payout calls 
# for each stash address will be 4). [default: 4]
CRUNCH_MAXIMUM_PAYOUTS=4
#
# [CRUNCH_SEED_PATH] File path containing the private seed phrase to Sign the extrinsic payout call. [default: .private.seed]
#CRUNCH_SEED_PATH=.private.seed.example
#
# Matrix configuration variables
CRUNCH_MATRIX_USER=@your-regular-matrix-account:matrix.org
CRUNCH_MATRIX_BOT_USER=@your-own-crunch-bot-account:matrix.org
CRUNCH_MATRIX_BOT_PASSWORD=anotthateasypassword
# 
# Default log
RUST_LOG="crunch=info"
```

Create a seed private file `.private.seed` inside `crunch-bot` folder and write the private seed phrase of the account responsible to Sign the extrinsic payout call [`.private.seed.example`](https://github.com/turboflakes/crunch/blob/main/.private.seed.example) (Note: `.private.seed` is the default name and hidden file, if you want something different you can adjust it with the option `--seed-path ~/crunch-bot/.kusama.private.seed` )

Note: vim adds a new line at the end by default, to prevent it use flag `vi -b file` and once in vim `:set noeol`
alternatively you can follow the steps [here](https://stackoverflow.com/questions/1050640/how-to-stop-vim-from-adding-a-newline-at-end-of-file)

```bash
#!/bin/bash
# create configuration file .env inside crunch-bot folder
$ touch ~/crunch-bot/.private.seed
# open file and write the private seed phrase of the account responsible to Sign the extrinsic payout call
$ vi -b ~/crunch-bot/.private.seed
# and than set no end line, write and quit
:set noeol
:wq!
```

By default just Run `crunch` inside `crunch-bot` folder

```bash
#!/bin/bash
# set ~/crunch-bot your current working directory
$ cd ~/crunch-bot
# simple run crunch with the generic options for Westend network
$ crunch westend flakes daily
```

```bash
#!/bin/bash
# or for the Kusama network claiming rewards every 6 hours
$ crunch kusama flakes turbo
# or for the Polkadot network claiming rewards once a day
$ crunch kusama flakes daily
```

## 🤖 Crunch Bot ([Matrix](https://matrix.org/))

To enable the matrix bot you will need to create an account on Element or similar and  adjust the respective environment variables `CRUNCH_MATRIX_BOT_USER` and `CRUNCH_MATRIX_BOT_PASSWORD` as specified in here [`.env.example`](https://github.com/turboflakes/crunch/blob/main/.env.example). You may also want to set your regular matrix user to the environment variable `CRUNCH_MATRIX_USER`. So that `crunch bot` could create a private room and send the messages in. By default `crunch bot` will automatically invite your regular matrix user to a private room and to a public room specific to the network which is connected to.

### Public rooms available

You can join the crew now and read messages history of all the **Crunch Bots** that send messages in to the following Public Rooms:

- [Westend Crunch Bot (Public)](https://matrix.to/#/%23westend-crunch-bot:matrix.org)
- [Kusama Crunch Bot (Public)](https://matrix.to/#/%23kusama-crunch-bot:matrix.org)
- [Polkadot Crunch Bot (Public)](https://matrix.to/#/%23polkadot-crunch-bot:matrix.org)

## Crunch [CLI] - Options

```bash
#!/bin/bash
$ ./target/debug/crunch -h

USAGE:
    crunch [OPTIONS] [CHAIN] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config-path <FILE>
            Sets a custom config file path. The config file contains 'crunch' configuration variables. [default: .env]

    -s, --stashes <stashes>
            Validator stash addresses for which 'crunch view', 'crunch flakes' or 'crunch rewards' will be applied. If
            needed specify more than one (e.g. stash_1,stash_2,stash_3).
    -w, --substrate-ws-url <substrate-ws-url>
            Substrate websocket endpoint for which 'crunch' will try to connect. (e.g. wss://kusama-rpc.polkadot.io)
            (NOTE: substrate_ws_url takes precedence than <CHAIN> argument)

ARGS:
    <CHAIN>    Sets the substrate-based chain for which 'crunch' will try to connect [default: westend]  [possible
               values: westend, kusama, polkadot]

SUBCOMMANDS:
    flakes     Crunch awesome flakes (rewards) daily or in turbo mode -> 4x faster [default subcommand]
    help       Prints this message or the help of the given subcommand(s)
    rewards    Claim staking rewards for unclaimed eras once a day or four times a day [default subcommand]
    view       Inspect staking rewards for the given stashes and display claimed and unclaimed eras.
```

Sub commands `crunch flakes` or `crunch rewards` are identical in terms of task execution. But logs, messages/notifications may differ, making **Crunch Bot** with `crunch flakes` a bit more fun!
```bash
#!/bin/bash
$ ./target/debug/crunch flakes -h

USAGE:
    crunch flakes [FLAGS] [OPTIONS] [MODE]

FLAGS:
        --debug                              Prints debug information verbosely.
        --disable-matrix                     Disable matrix bot for 'crunch flakes'. (e.g. with this flag active 'crunch
                                             flakes' will not send messages/notifications about claimed or unclaimed
                                             staking rewards to your private or public 'Crunch Bot' rooms)
                                             (https://matrix.org/)
        --disable-matrix-bot-display-name    Disable matrix bot display name update for 'crunch flakes'. (e.g. with this
                                             flag active 'crunch flakes' will not change the matrix bot user display
                                             name)
        --disable-public-matrix-room         Disable notifications to matrix public rooms for 'crunch flakes'. (e.g.
                                             with this flag active 'crunch flakes' will not send messages/notifications
                                             about claimed or unclaimed staking rewards to any public 'Crunch Bot' room)
    -h, --help                               Prints help information
    -V, --version                            Prints version information

OPTIONS:
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
    <MODE>    Sets how often flakes (staking rewards) should be crunched (claimed) from unclaimed eras. (e.g. the
              option 'daily' sets 'crunch' task to be repeated every 24 hours; option 'turbo' sets 'crunch' task to
              be repeated every 6 hours) [default: turbo]  [possible values: daily, turbo]
```

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

## About

Crunch was made by <a href="https://turboflakes.com" target="_blank">TurboFlakes</a>.

TurboFlakes is also an independent validator in the Kusama Network and eligible in the Kusama's Thousand Validators Programme, aka <a href="https://thousand-validators.kusama.network/#/leaderboard/FZsMKYHoQG1dAVhXBMyC7aYFYpASoBrrMYsAn1gJJUAueZX" target="_blank" rel="noreferrer">1KV</a>.

If you like this project ✌️ Share our work and support us with your nomination or tip ✨💙

- **Polkadot**: 12gPFmRqnsDhc9C5DuXyXBFA23io5fSGtKTSAimQtAWgueD2
- **Kusama**: FZsMKYHoQG1dAVhXBMyC7aYFYpASoBrrMYsAn1gJJUAueZX

You could also Star the Github project or make a pull request for a new feature. 

In case you find a bug, please open a Github issue [here](https://github.com/turboflakes/crunch/issues).
