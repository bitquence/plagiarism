# Plagiarism

Plagiarism is a small Rust script used to relay messages from one channel into another using a user account and a webhook (or otherwise **mirror** incoming messages).

## Usage

To use Plagiarism, you will need to obtain your user account's token. For every channel you wish to relay, you will need its channel ID and a webhook to relay the messages to.

Once you have these, you can populate a configuration file named `config.json` with the following JSON object:

```json
{
    "token": "USER_ACCOUNT_TOKEN",
    "channels": {
        "CHANNEL_ID": "WEBHOOK_URL"
    }
}
```

To enable logging, you will need to make a file named `.env` and populate it with the following configuration:

```sh
RUST_LOG=RUST_LOG=off,plagiarism=info
RUST_BACKTRACE=full
```

Upon running, the program will start listening for messages sent to any of the channels listed. Any new messages found in the channel will be relayed to its corresponding webhook.

## Modifications to serenity

The version of the [serenity](https://github.com/serenity-rs/serenity) crate which can be found [here](https://github.com/bitquence/serenity) has been modified to use the same payload a regular browser would when connecting to Discord's gateway. Additionally, it was modified to allow the use of a user account instead of a bot account, which the original no longer allows.

## ⚠ Warning

This program automates a user account which is forbidden by [Discord's Terms of Service](https://support.discord.com/hc/en-us/articles/115002192352-Automated-user-accounts-self-bots-), using it may result in termination of your account. Use it at your own risk.