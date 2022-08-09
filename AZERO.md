# Notes for Aleph Zero users
<<<<<<< HEAD

Crunch now support both Aleph Zero chains: use `azero` to connect to mainnet and `tzero` to connet to testnet

While Aleph Zero evolves, some functionalities may diverge from the standard substrate framework, thus please have the following topics in mind when configuring Crunch for your Flakes.
=======
While Aleph Zero evolves, some functionalities may diverge from the standard substrate framework, thus please have the following topics in mind when configuring Crunch for your networks.
>>>>>>> 457dba90233e85b83ce8d378c197e56fe62e59da

## Quick start
After installation (according to README.md), you can quickly try the connection to the relevant network (i.e. azero or tzero) using the following command:

    crunch -stashes <YOUR_STASH> azero view

If everything goes to plan, you will receive something similar to the following results:

<<<<<<< HEAD
> [INFO  crunch] crunch v0.5.5 * Crunch is a command-line interface (CLI) to claim staking rewards (flakes) every X hours for Substrate-based chains
> [INFO  crunch::crunch] Connected to Aleph Zero network using wss://ws.azero.dev:443 * Substrate node Substrate Node v0.4.0-4ab787d-x86_64-linux-gnu
=======
> [INFO  crunch] crunch v0.5.5 * Crunch is a command-line interface (CLI) to claim staking rewards (flakes) every X hours for Substrate-based chains [INFO  crunch::crunch] Connected to Aleph Zero network using wss://ws.azero.dev:443 * Substrate node Substrate Node v0.4.0-4ab787d-x86_64-linux-gnu
>>>>>>> 457dba90233e85b83ce8d378c197e56fe62e59da
> [ERROR crunch::crunch] matrix bot user '' does not specify the matrix server e.g. '@your-own-crunch-bot-account:matrix.org'
> [INFO crunch::runtimes::aleph_zero] Inspect stashes -> 5GFCZjWGSHas86192H3yiZZFySLtUW74SdHDqTymBEDUUF7T
> [INFO crunch::runtimes::aleph_zero] 5GFCZjWGSHas86192H3yiZZFySLtUW74SdHDqTymBEDUUF7T * Stash account
> [INFO crunch::runtimes::aleph_zero] 83 claimed eras in the last 84 -> [20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 75, 76, 77, 78, 79, 80, 81, 82, 83, 84, 85, 86, 87, 88, 89, 90, 91, 92, 93, 94, 95, 96, 97, 98, 99, 100, 101, 102]
> [INFO crunch::runtimes::aleph_zero] 1 unclaimed eras in the last 84 -> [103]
> [INFO  crunch::runtimes::aleph_zero] Job done!

Note the error code generated when you have not specified a matrix / Element account. In order to correct that read the following title:

## Element account
Crunch is most useful when you are able to receive notifications of its results in your devices. Please consider creating one Element account for you and one for your Crunch bot. The details of both accounts can be added to the configuration `.env` file.

> Note: [Element](https://element.io/) is a a free and open-source instant messaging client implementing the [Matrix](https://matrix.org/) protocol.

If you prefer not to use the notifications feature at all, just pass the `--disable-matrix` flag to the Crunch command, e.g.:

    crunch --config-path <YOUR.ENV.FILEPATH> azero rewards era --disable-matrix

Else, if you only want to receive notifications yourself, but avoid the bot to send a copy to the public room, use:

    crunch --config-path <YOUR.ENV.FILEPATH> azero rewards era --disable-public-matrix-room

## On-chain Identity
Although already available in testnet, the Identity pallet is still to be deployed in the mainnet, until then, you will notice that no account names are shown by Crunch.

## Configuration details
For Crunch to efficiently process your flakes (rewards), please note the following recommendations for your `.env` configuration file:

Use the websocket endpoint of your own (non-validating) node to connect to the network. Leaving the Foundation's public websocket (`wss://ws.azero.dev`) may cause excessive strain on the public services (i.e. to the `azero.dev` wallet):

    CRUNCH_SUBSTRATE_WS_URL=ws://localhost:9944

Also, please note that both testnet and mainnet seems to only accept single calls (instead of allowing batch calls), thus please adjust the following variable accordingly:

    CRUNCH_MAXIMUM_CALLS=1

<<<<<<< HEAD
That's it!!

Thanks for using Crunch and enjoy your Flakes!!
=======
That's it!!  Thanks for using Crunch and enjoy your Flakes!!

> Written with [StackEdit](https://stackedit.io/).

>>>>>>> 457dba90233e85b83ce8d378c197e56fe62e59da
