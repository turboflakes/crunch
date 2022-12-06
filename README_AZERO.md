# Notes for Aleph Zero users
Crunch now support both Aleph Zero chains: use `azero` to connect to mainnet and `tzero` to connect to testnet

>   While Aleph Zero evolves, some functionalities may diverge from the standard substrate framework, thus please have the following topics in mind when configuring Crunch for your Flakes.

## Quick start
After installation (according to README.md), you can quickly try the connection to the relevant network (i.e. azero or tzero) using the following command:

    crunch --stashes <YOUR_STASH> azero view

If everything goes to plan, you will receive something similar to the following results:

    [INFO  crunch] crunch v0.6.1 * Crunch is a command-line interface (CLI) to claim staking rewards (flakes) every X hours for Substrate-based chains
    [INFO  crunch::crunch] Connected to Aleph Zero network using wss://ws.azero.dev:443 * Substrate node Substrate Node v0.4.0-4ab787d-x86_64-linux-gnu
    [ERROR crunch::crunch] matrix bot user '' does not specify the matrix server e.g. '@your-own-crunch-bot-account:matrix.org'
    [INFO  crunch::runtimes::aleph_zero] Inspect stashes -> 5Ggatomsdkh6ByjPUfrHvEhdGpGNFh8rai1cdzokJz3KvgD1
    [INFO  crunch::runtimes::aleph_zero] 5Ggatomsdkh6ByjPUfrHvEhdGpGNFh8rai1cdzokJz3KvgD1 * Stash account
    [INFO  crunch::runtimes::aleph_zero] 0 claimed eras in the last 0 -> []
    [INFO  crunch::runtimes::aleph_zero] 0 unclaimed eras in the last 0 -> []
    [INFO  crunch::runtimes::aleph_zero] Job done!

Note the error code generated when you have not specified a Matrix / Element account. Thus, it is of utmost importance that you run your payout routine as shown below:

## IMPORTANT: COMMAND SYNTAX
At the moment, no public Matrix room is configured for the networks of Aleph Zero, so please remember to use the flag `--disable-public-matrix-room` when you run the payout routine:

    crunch --config-path <YOUR_ENV_FILE_PATH> rewards era --disable-public-matrix-room

## IMPORTANT: ENV FILE
For Crunch to efficiently process your flakes (rewards), please note the following recommendations for your `.env` configuration file:

Use the websocket endpoint of your own (non-validator) node to connect to the network. Leaving the Foundation's public websocket (`wss://ws.azero.dev`) may cause excessive strain on the public services (i.e. to the `azero.dev` wallet):

    CRUNCH_SUBSTRATE_WS_URL=ws://localhost:9944

Also, please note that both testnet and mainnet seems to only accept single calls (instead of allowing batch calls), thus please adjust the following variable accordingly:

    CRUNCH_MAXIMUM_CALLS=1

## Matrix account
Crunch is most useful when you are able to receive notifications of its results in your devices. Please consider creating one Matrix account for you and one for your Crunch bot. The details of both accounts can be added to the configuration `.env` file.

> Note: [Element](https://element.io/) is a free and open-source instant messaging client implementing the [Matrix](https://matrix.org/) protocol.

If you prefer not to use the notifications feature at all, just pass the `--disable-matrix` flag to the Crunch command, e.g.:

    crunch --config-path <YOUR_ENV_FILE_PATH> azero rewards era --disable-matrix

Else, if you only want to receive notifications yourself, but avoid the bot to send a copy to the public room, use `--disable-public-matrix-room`:

    crunch --config-path <YOUR_ENV_FILE_PATH> azero rewards era --disable-public-matrix-room


Thanks for using _*Crunch*_ and enjoy your _*Flakes*_!!
