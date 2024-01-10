# Gluwa Related Info

Information related to Gluwa specific changes for Crunch.

## Metadata generation 

```shell
subxt metadata --version 14 -f bytes > metadata/creditcoin_metadata.scale
```

Or use the handy `gen_metadata.sh` script. 

## How to update docker image when metadata changes. 

Create a new branch. Run a local version of your node with the updated runtime and run the `gen_metadata.sh` script. Move that metadata to the `metadata` folder and check that the binary compiles (Because subxt uses macros you need to perform a full compile and not just a `cargo check` for completeness). After you test the new metadata create a PR and merge your branch into either `mainnet`, `testnnet`, or `devnet`. A push to either of these branches triggers a workflow that builds the docker image and pushes it to Gluwa's repo. The image will be named `crunch-[BRANCH]:latest`. 

## runtimes/creditcoin.rs

This file is almost identical to `runtimes/polkadot.rs`. The only changes made were to the path of the metadata file for the node_runtime and in the function `try_run_batch_pool_members`. The `member` field expected an Address32 according to the compiler and it the other implementations it was a `MultiAddress`.

## Issues with M1 Macs

If you have issues building the docker image for this repo and you are on an M1 mac try upgrading your version of MacOs. YMMV 

# crunch &middot; ![latest release](https://github.com/turboflakes/crunch/actions/workflows/create_release.yml/badge.svg)

<p align="center">
  <img src="https://github.com/turboflakes/crunch/blob/main/assets/crunchbot-github-header.png?raw=true">
</p>

`crunch` is a command-line interface (CLI) to easily automate payouts of staking rewards on Substrate-based chains.

## Why use `crunch`

To claim staking rewards for just one or a list of Validators at the end of each Era or every X hours.

To get notified about the amount and rate of the total staking rewards each Validator and their Nominators got

To be informed about era stats for each Validator, e.g. inclusion rate, claimed rewards rate, era points trend, active for current era

To easily inspect about any unclaimed eras for a given Validator stash

To promote Validators by publicly publish their automated staking rewards to a public **Crunch Bot** room

For Nominators in private or public rooms check their chosen Validators rewards performance

For Pool Operators to ensure that all active validators selected - active pool nominees - payout on time. An additional guarantee for Pool members that Pool operatos act in members best interests.

For Pool Operators to auto-compound members rewards above certain threshold.

## Installation

```bash
#!/bin/bash
# create `crunch-bot` directory
mkdir /crunch-bot
# download `crunch` binary latest version
wget -P /crunch-bot https://github.com/turboflakes/crunch/releases/download/v0.10.1/crunch
# make `crunch` binary file executable
chmod +x /crunch-bot/crunch
```

Note: Alternatively download [`crunch-update.sh`](https://github.com/turboflakes/crunch/blob/main/crunch-update.sh) bash script file and make it executable. Easier installation and faster updates.

## Configuration

Create a configuration file `.env` inside `crunch-bot` folder and copy the default variables from [`.env.example`](https://github.com/turboflakes/crunch/blob/main/.env.example) (Note: `.env` is the default name and a hidden file, if you want something different you can adjust it later with the option `crunch --config-path /crunch-bot/config_kusama.env` )

```bash
#!/bin/bash
# create/open a file with a file editor (Vim in this case) and add/change the configuration
# variables with your own personal values
vi /crunch-bot/.env
# when ready write and quit (:wq!)
```

Configuration file example: [`.env.example`](https://github.com/turboflakes/crunch/blob/main/.env.example)

```bash
# ----------------------------------------------------------------
# crunch CLI configuration variables 
# ----------------------------------------------------------------
# [CRUNCH_STASHES] Validator stash addresses for which 'crunch flakes', 'crunch rewards'
# or 'crunch view' will be applied. 
# If needed specify more than one (e.g. stash_1,stash_2,stash_3).
CRUNCH_STASHES=5GTD7ZeD823BjpmZBCSzBQp7cvHR1Gunq7oDkurZr9zUev2n
#
# [CRUNCH_STASHES_URL] Additionally the list of stashes could be defined and available in a remote file.
# `crunch` will try to fetch the stashes from the endpoint predefined here before triggering the respective payouts
# Please have a look at the file '.remote.stashes.example' as an example
CRUNCH_STASHES_URL=https://raw.githubusercontent.com/turboflakes/crunch/main/.remote.stashes.example
#
# [CRUNCH_SUBSTRATE_WS_URL] Substrate websocket endpoint for which 'crunch' will try to
# connect. (e.g. wss://kusama-rpc.polkadot.io) (NOTE: substrate_ws_url takes precedence
# than <CHAIN> argument) 
#CRUNCH_SUBSTRATE_WS_URL=wss://westend-rpc.polkadot.io:443
#
# [CRUNCH_MAXIMUM_PAYOUTS] Maximum number of unclaimed eras for which an extrinsic payout
# will be submitted. (e.g. a value of 4 means that if there are unclaimed eras in the last
# 84 the maximum unclaimed payout calls for each stash address will be 4). [default: 4]
CRUNCH_MAXIMUM_PAYOUTS=4
#
# [CRUNCH_MAXIMUM_HISTORY_ERAS] Maximum number of history eras for which crunch will look for 
# unclaimed rewards. The maximum value supported is the one defined by constant history_depth
# (e.g. a value of 4 means that crunch will only check in the latest 4 eras if there are any 
# unclaimed rewards for each stash address). [default: 4]
CRUNCH_MAXIMUM_HISTORY_ERAS=4
#
# [CRUNCH_MAXIMUM_CALLS] Maximum number of calls in a single batch. [default: 8]
CRUNCH_MAXIMUM_CALLS=8
#
# [CRUNCH_SEED_PATH] File path containing the private seed phrase to Sign the extrinsic 
# payout call. [default: .private.seed]
#CRUNCH_SEED_PATH=.private.seed.example
# ----------------------------------------------------------------
# Matrix configuration variables
# ----------------------------------------------------------------
CRUNCH_MATRIX_USER=@your-regular-matrix-account:matrix.org
CRUNCH_MATRIX_BOT_USER=@your-own-crunch-bot-account:matrix.org
# NOTE: type the bot password within "" so that any special character could be parsed correctly into a string.
CRUNCH_MATRIX_BOT_PASSWORD="anotthateasypassword"
# ----------------------------------------------------------------
# ONE-T configuration variables
# ----------------------------------------------------------------
CRUNCH_ONET_API_ENABLED=true
CRUNCH_ONET_API_URL=https://kusama-onet-api-beta.turboflakes.io
CRUNCH_ONET_API_KEY=crunch-101
CRUNCH_ONET_NUMBER_LAST_SESSIONS=6
# ----------------------------------------------------------------
# Nomination Pools configuration variables
# ----------------------------------------------------------------
# [CRUNCH_POOL_IDS] Additionally the list of stashes could be defined from a single or more Nomination Pool Ids.
# `crunch` will try to fetch the nominees of the respective pool id predefined here before triggering the respective payouts
CRUNCH_POOL_IDS=10,15
#
# [CRUNCH_POOL_COMPOUND_THRESHOLD] Define minimum pending rewards threshold in PLANCKS. 
# Note: only pending rewards above the threshold are included in the auto-compound batch.
CRUNCH_POOL_COMPOUND_THRESHOLD=100000000000
#
# [CRUNCH_POOL_MEMBERS_COMPOUND_ENABLED] Enable auto-compound rewards for every member that belongs to the pools 
# previously selected by CRUNCH_POOL_IDS. Note that members have to have their permissions 
# set as PermissionlessCompound or PermissionlessAll.
#CRUNCH_POOL_MEMBERS_COMPOUND_ENABLED=true
#
# [CRUNCH_POOL_ONLY_OPERATOR_COMPOUND_ENABLED] Enable auto-compound rewards for the pool operator member that belongs to the pools 
# previously selected by CRUNCH_POOL_IDS. Note that operator member account have to have their permissions 
# set as PermissionlessCompound or PermissionlessAll.
CRUNCH_POOL_ONLY_OPERATOR_COMPOUND_ENABLED=true
#
# [CRUNCH_POOL_ACTIVE_NOMINEES_PAYOUT_ENABLED] Enable payouts only for ACTIVE nominees assigned to the pools 
# previously selected by CRUNCH_POOL_IDS.
#CRUNCH_POOL_ACTIVE_NOMINEES_PAYOUT_ENABLED=true
#
# [CRUNCH_POOL_ALL_NOMINEES_PAYOUT_ENABLED] Enable payouts for ALL nominees assigned to the pools 
# previously selected by CRUNCH_POOL_IDS.
#CRUNCH_POOL_ALL_NOMINEES_PAYOUT_ENABLED=true
```

Create a seed private file `.private.seed` inside `crunch-bot` folder and write the private seed phrase of the account responsible to sign the extrinsic payout call as in [`.private.seed.example`](https://github.com/turboflakes/crunch/blob/main/.private.seed.example) (Note: `.private.seed` is the default name and a hidden file, if you want something different you can adjust it later with the option `crunch flakes --seed-path ~/crunch-bot/.kusama.private.seed` )

```bash
#!/bin/bash
# create a file with a file editor (Vim in this case) and write the private seed phrase 
# of the account responsible to sign the extrinsic payout call
vi /crunch-bot/.private.seed
# when ready write and quit (:wq!)
```

### Configuration of _systemd_ service

A good idea is to run the tool as a `systemd` service. Based on the previous path configuration, here is an example for reference:

```bash
[Unit]
Description=Kusama Crunch Bot

[Service]
ExecStart=/crunch-bot/crunch --config-path /crunch-bot/.env rewards era --seed-path /crunch-bot/.private.seed
Restart=always
RestartSec=15

[Install]
WantedBy=multi-user.target

```

### Crunch Bot ([Matrix](https://matrix.org/))

If you set up `crunch` on your server with a matrix user 👉  you get your own **Crunch Bot**.

To enable **Crunch Bot** you will need to create a specific account on Element or similar and  copy the values to the respective environment variables `CRUNCH_MATRIX_BOT_USER` and `CRUNCH_MATRIX_BOT_PASSWORD` like in the configuration example file [`.env.example`](https://github.com/turboflakes/crunch/blob/main/.env.example). You may also want to set your regular matrix user to the environment variable `CRUNCH_MATRIX_USER`. So that **Crunch Bot** could create a private room and send in messages. By default **Crunch Bot** will automatically invite your regular matrix user to a private room. Also by default **Crunch Bot** will send a copy of the messages to the respective network public room for which is connected to.

### Public Rooms available

Join and read the messages history of all the Public Rooms for which **Crunch Bots** are sending messages:

<table style="width:100%;" cellspacing="0" cellpadding="0">
  <tr>
    <td style="width: 100px;">
        <img style="width: 80px;" src="https://github.com/turboflakes/crunch/blob/main/assets/crunchbot-westend-room-128.png?raw=true" />
    </td>
    <td><a href="https://matrix.to/#/%23westend-crunch-bot:matrix.org" target="_blank">Westend Crunch Bot (Public)</a></td>
  </tr>
  <tr>
    <td style="width: 100px;">
        <img style="width: 80px;" src="https://github.com/turboflakes/crunch/blob/main/assets/crunchbot-kusama-room-128.png?raw=true" />
    </td>
    <td><a href="https://matrix.to/#/%23kusama-crunch-bot:matrix.org" target="_blank">Kusama Crunch Bot (Public)</a></td>
  </tr>
  <tr>
    <td style="width: 100px;">
        <img style="width: 80px;" src="https://github.com/turboflakes/crunch/blob/main/assets/crunchbot-polkadot-room-128.png?raw=true" />
    </td>
    <td><a href="https://matrix.to/#/%23polkadot-crunch-bot:matrix.org" target="_blank">Polkadot Crunch Bot (Public)</a></td>
  </tr>
</table>

### Crunch Bot messages

![crunch bot notification messages example](https://github.com/turboflakes/crunch/blob/main/assets/matrix-example-512.png?raw=true)

## Usage

If you have been doing `crunch` configuration as described in previous steps (assuming `.env` and `.private.seed` defined inside `/crunch-bot` folder), run `crunch` when `/crunch-bot` folder is your current working directory. Otherwise you will have to specify `.env` and `.private.seed` custom paths.

```bash
#!/bin/bash
# set /crunch-bot your current working directory
cd /crunch-bot
```

By default `crunch` tries to connect to your local substrate node on the default websocket port `ws://127.0.0.1:9944`. This can be changed by typing one of polkadot main chains - westend, kusama or polkadot. Or by changing the substrate websocket url with the option `--substrate-ws-url`.

`crunch` default sub command is `flakes`, there are fun messages if you stick with it, or you can choose the regular sub command `rewards` rather than `flakes`. As you prefer. Both sub commands are identical in terms of job execution. But logs, messages/notifications differ.

Essentially `crunch` motto is enjoy **Crunch Bot** while `crunch flakes` :)

If all has been set as previously described `crunch` should be ready with just the following options:

```bash
#!/bin/bash
# if running a local node than simple run crunch with default options
# by default crunch will try to connect to ws://localhost:9944
# and claim staking rewards as soon as the current era finishes
crunch rewards
# or be specific to which network crunch will try to connect
crunch kusama rewards
# or for Polkadot network and claiming rewards once a day at a specific time
crunch polkadot rewards daily
# or for Westend network and claiming rewards every 6 hours at a specific time
crunch westend rewards turbo
# or for Westend network with unique stashes verified and for all configured pools nominees and claiming rewards every era
crunch westend --enable-unique-stashes rewards era --enable-pool-all-nominees-payout
# or to auto-compound members rewards of nomination pools you operate
crunch kusama rewards --enable-pool-members-compound
# or to know which ONE-T grade a validator got from the last 6 sessions
crunch kusama rewards --enable-onet-api
# or try flakes just for fun :)
crunch flakes
# to list all options try help
crunch help
```

If you need more customization run help to check all sub commands, flags and options.

Note: All flags and options are also available through environment variables if defined in `.env` configuration file. You can choose which way you want to configure `crunch`. Take in consideration that if the same variable is defined on both sides e.g. defined in `.env` and through CLI flag/option, `crunch` will take the value defined by CLI.

```bash
#!/bin/bash
# if you need a custom crunch check all the options and flags available
crunch help

USAGE:
    crunch [FLAGS] [OPTIONS] [CHAIN] [SUBCOMMAND]

FLAGS:
        --enable-unique-stashes    From all given stashes crunch will Sort by stash adddress and Remove duplicates.
    -h, --help                     Prints help information
    -V, --version                  Prints version information

OPTIONS:
    -c, --config-path <FILE>
            Sets a custom config file path. The config file contains 'crunch' configuration variables. [default: .env]

    -s, --stashes <stashes>
            Validator stash addresses for which 'crunch view', 'crunch flakes' or 'crunch rewards' will be applied. If
            needed specify more than one (e.g. stash_1,stash_2,stash_3).
        --stashes-url <stashes-url>
            Remote stashes endpoint for which 'crunch' will try to fetch the validator stash addresses (e.g.
            https://raw.githubusercontent.com/turboflakes/crunch/main/.remote.stashes.example).
    -w, --substrate-ws-url <substrate-ws-url>
            Substrate websocket endpoint for which 'crunch' will try to connect. (e.g. wss://kusama-rpc.polkadot.io)
            (NOTE: substrate_ws_url takes precedence than <CHAIN> argument)

ARGS:
    <CHAIN>    Sets the substrate-based chain for which 'crunch' will try to connect [possible values: westend,
               kusama, polkadot]

SUBCOMMANDS:
    flakes     Crunch awesome flakes (rewards) every era, daily or in turbo mode -> 4x faster
    help       Prints this message or the help of the given subcommand(s)
    rewards    Claim staking rewards for unclaimed eras once a day or four times a day [default subcommand]
    view       Inspect staking rewards for the given stashes and display claimed and unclaimed eras.
```

```bash
#!/bin/bash
# or help for any subcommand like
crunch rewards --help

USAGE:
    crunch rewards [FLAGS] [OPTIONS] [MODE]

FLAGS:
        --debug                                 Prints debug information verbosely.
        --disable-matrix                        Disable matrix bot for 'crunch rewards'. (e.g. with this flag active
                                                'crunch rewards' will not send messages/notifications about claimed or
                                                unclaimed staking rewards to your private or public 'Crunch Bot' rooms)
                                                (https://matrix.org/)
        --disable-matrix-bot-display-name       Disable matrix bot display name update for 'crunch rewards'. (e.g. with
                                                this flag active 'crunch rewards' will not change the matrix bot user
                                                display name)
        --disable-public-matrix-room            Disable notifications to matrix public rooms for 'crunch rewards'. (e.g.
                                                with this flag active 'crunch rewards' will not send
                                                messages/notifications about claimed or unclaimed staking rewards to any
                                                public 'Crunch Bot' room)
        --enable-onet-api                       Allow 'crunch' to fetch grades for every stash from ONE-T API.
        --enable-pool-active-nominees-payout    Enable payouts only for ACTIVE nominees assigned to the Nomination Pools
                                                defined in 'pool-ids'. (e.g. with this flag active 'crunch' will try to
                                                trigger payouts only for the ACTIVE nominees and not all).
        --enable-pool-all-nominees-payout       Enable payouts for ALL the nominees assigned to the Nomination Pools
                                                defined in 'pool-ids'. (e.g. with this flag active 'crunch' will try to
                                                trigger payouts for ALL nominees and not only the active ones - the ones
                                                the stake of the Nomination Pool was allocated).
        --enable-pool-members-compound          Allow 'crunch' to compound rewards for every member that belongs to the
                                                pools previously selected by '--pool-ids' option. Note that members have
                                                to have their permissions set as PermissionlessCompound or
                                                PermissionlessAll.
        --enable-pool-only-operator-compound    Allow 'crunch' to compound rewards for the pool operator member that
                                                belongs to the pools previously selected by '--pool-ids' option. Note
                                                that the operator member account have to have their permissions set as
                                                PermissionlessCompound or PermissionlessAll.
    -h, --help                                  Prints help information
        --short                                 Display only essential information (e.g. with this flag active 'crunch
                                                rewards' will only send essential messages/notifications about claimed
                                                rewards)
    -V, --version                               Prints version information

OPTIONS:
        --enable-pool-compound-threshold <enable-pool-compound-threshold>
            Define minimum pending rewards threshold in PLANCKS. (e.g. Only pending rewards above the threshold are
            include in the auto-compound batch)
        --error-interval <error-interval>
            Interval value (in minutes) from which 'crunch' will restart again in case of a critical error.

        --matrix-bot-password <matrix-bot-password>
            Password for the 'Crunch Bot' matrix user sign in.

        --matrix-bot-user <matrix-bot-user>
            Your new 'Crunch Bot' matrix user. e.g. '@your-own-crunch-bot-account:matrix.org' this user account will be
            your 'Crunch Bot' which will be responsible to send messages/notifications to your private or public 'Crunch
            Bot' rooms.
        --matrix-user <matrix-user>
            Your regular matrix user. e.g. '@your-regular-matrix-account:matrix.org' this user account will receive
            notifications from your other 'Crunch Bot' matrix account.
        --maximum-calls <maximum-calls>
            Maximum number of calls in a single batch. [default: 8]

        --maximum-history-eras <maximum-history-eras>
            Maximum number of history eras for which crunch will look for unclaimed rewards. The maximum value supported
            is the one defined by the constant history_depth - usually 84 - (e.g. a value of 4 means that crunch will
            only check in latest 4 eras if there are any unclaimed rewards for each stash address). [default: 4]
    -m, --maximum-payouts <maximum-payouts>
            Maximum number of unclaimed eras for which an extrinsic payout will be submitted. (e.g. a value of 4 means
            that if there are unclaimed eras in the last 84 the maximum unclaimed payout calls for each stash address
            will be 4).
        --pool-ids <pool-ids>
            Nomination pool ids for which 'crunch' will try to fetch the validator stash addresses (e.g. poll_id_1,
            pool_id_2).
    -f, --seed-path <FILE>
            Sets a custom seed file path. The seed file contains the private seed phrase to Sign the extrinsic payout
            call.

ARGS:
    <MODE>    Sets how often staking rewards should be claimed from unclaimed eras. (e.g. the option 'era' sets
              'crunch' task to run as soon as the EraPaid on-chain event is triggered; the option 'daily' sets
              'crunch' task to be repeated every 24 hours; option 'turbo' sets 'crunch' task to be repeated every 6
              hours) [default: era]  [possible values: era, daily, turbo]
```

Note: By default `crunch` collects the outstanding payouts from previous eras and group all the extrinsic payout calls in group of 4 or whatever value defined in the flag `maximum-calls` so that a single batch call per group can be made. The collection of all outstanding payouts from previous eras is also limited by 2 other flags. The first being `maximum-payouts` which default value is 4, this flag limits the number of payouts **per stash**. The other one is the `maximum-history-eras` which default is also 4, this flag limits the number of past eras `crunch` will look for unclaimed rewards - but this flag only applies if `short` flag is also used in the configuration. This is done so that `crunch` can run efficiently every era.

With that said, if it's the **first time** you are running `crunch` and you are not sure if you have any unclaimed rewards or if you just want to know for the stash accounts defined in the confguration file (`.env`), which eras from the last 84 have already been claimed or unclaimed, you can simply run `crunch view`.

Note: The `crunch view` mode only logs information into the terminal.

```bash
#!/bin/bash
# log unclaimed rewards for Westend network 
crunch westend view
# or for Kusama network
crunch kusama view
# or for Polkadot network
crunch polkadot view
```

Note: You can run `crunch` inside a tmux session and leave it, or using something like `systemd` to run `crunch` on server restarts for example. By default `crunch` will wake up every X hours to claim rewards if there are any to claim.

## Common issue on Ubuntu 22.04 when using the crunch binary

Install previous openssl version from:
```
wget http://archive.ubuntu.com/ubuntu/pool/main/o/openssl/libssl1.1_1.1.1f-1ubuntu2.20_amd64.deb
dpkg -i libssl1.1_1.1.1f-1ubuntu2.20_amd64.deb
```

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
git clone http://github.com/turboflakes/crunch
```

Compile `crunch` package with Cargo

```bash
#!/bin/bash
cargo build
```

And then run it

```bash
#!/bin/bash
./target/debug/crunch westend flakes daily
```

Otherwise, recompile the code on changes and run the binary

```bash
#!/bin/bash
cargo watch -x 'run --bin crunch'
```

### Downloading metadata from a Substrate node

Use the [`subxt-cli`](./cli) tool to download the metadata for your target runtime from a node.

Install
```bash
cargo install subxt-cli
```
Save the encoded metadata to a file
```bash
subxt metadata --url https://westend-rpc.polkadot.io  -f bytes > westend_metadata.scale
```
(Optional) Generate runtime API client code from metadata
```bash
subxt codegen --url https://westend-rpc.polkadot.io | rustfmt --edition=2018 --emit=stdout > westend_runtime.rs
```

## Docker

`crunch` can also be built and run from a Docker container.

Build Image

```bash
git clone http://github.com/turboflakes/crunch
cd crunch
docker build -t localhost/crunch .
```

Run Container

```bash
docker run --rm -it localhost/crunch:latest --version
```

The config and seed files can be mounted from the host. Any options supported by `crunch` can be added.

```bash
docker run --rm -it \
  --volume=/etc/crunch/env:/.env:U \
  --volume=/etc/crunch/private.seed:/.private.seed:U \
  localhost/crunch:latest \
    rewards era
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

`crunch` was made by **TurboFlakes**. Visit us at <a href="https://turboflakes.io" target="_blank" rel="noreferrer">turboflakes.io</a> to know more about our work.

If you like this project
  - 🚀 Share our work 
  - ✌️ Visit us at <a href="https://turboflakes.io" target="_blank" rel="noreferrer">turboflakes.io</a>
  - ✨ Or you could also star the Github project :)

Tips are welcome

- Polkadot 14Sqrs7dk6gmSiuPK7VWGbPmGr4EfESzZBcpT6U15W4ajJRf (turboflakes.io)
- Kusama H1tAQMm3eizGcmpAhL9aA9gR844kZpQfkU7pkmMiLx9jSzE (turboflakes.io)

### License

`crunch` - The entire code within this repository is licensed under the [Apache License 2.0](./LICENSE).

### Quote

> "Study hard what interests you the most in the most undisciplined, irreverent and original manner possible."
― Richard Feynmann

__

Enjoy `crunch`
