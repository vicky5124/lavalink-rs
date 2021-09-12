# Lavalink-rs

A `lavalink` API wrapping library for every `tokio` discord crate.

## Links to download stuff you will need

- [Lavalink repository](https://github.com/freyacodes/Lavalink)
- [Java download](https://adoptopenjdk.net/) (11 or newer, 15 recommended, OpenJ9 works)

## TODO

- [ ] Support multiple connections per region.
- [X] Support nodes.
- [ ] Support actual nodes.
- [X] Switch to internal lock handles.
- [ ] Support identifiers.
- [X] Support pause, resume, skip to time.
- [X] Support starting at specific times and configurable replace current stream.
- [X] Support equalization.
- [X] Support both rustls and native_tls backends as features.
- [X] Support twilight.
- [X] Support events.
- [ ] Support raw events.
- [X] Implement my own event handler for voice connections.
- [ ] Support Sharding for the discord gateway.
- [X] Support easy queues natively.
- [X] Optimize the codebase.
- [X] Remove all the clones from examples.
- [X] Improve error handling.
- [X] Add tracing and logging.
- [X] Add documentation.
- [ ] Implement automatic reconnecting.
- [X] Make a ClientBuilder

## How to use

The minimum required Rust version is 1.51 due to a dependency of Songbird.

Install the version from crates.io:

```toml
lavalink-rs = "0.9-rc"
# or
[dependencies.lavalink-rs]
version = "0.9-rc"
```

Or the development release:

```toml
lavalink-rs = { git = "https://gitlab.com/vicky5124/lavalink-rs/", branch = "master"}
# or
[dependencies.lavalink-rs]
git = "https://gitlab.com/vicky5124/lavalink-rs/"
branch = "master"
```

If you wish to use a development version of serenity, add the following to the Cargo.toml:

```toml
[patch.crates-io.serenity]
git = "https://github.com/serenity-rs/serenity"
branch = "next"
```

### Features

Default features are: `rustls` and `songbird`
These are the available ones:

- `rustls`: Use the rustls TLS backend.
- `native`: Uses the system native TLS backend (OpenSSL on linux).
- `songbird`: Use songbird to connect to handle voice connections.
- `simple-gateway`: Use lavalink-rs to handle the voice connections (note, this is a very basic implementation, without sharding support, while also creating a second gateway rather than using the existing one).
- `wait-before-connect`: Use this feature if you want the lavalink-rs simple gateway to wait 6 seconds before connecting.
- `serenity`: Add support for serenity's models.
- `twilight`: Add support for twilight-model.
