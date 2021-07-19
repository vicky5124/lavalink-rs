# Lavalink-rs

A `lavalink` and `andesite` API wrapping library for every `tokio` discord crate.

## Links to download stuff you will need

- [Lavalink repository](https://github.com/freyacodes/Lavalink)
- [Andesite Repository](https://github.com/natanbc/andesite)
- [Java download](https://adoptopenjdk.net/) (11 or newer, 15 recommended, OpenJ9 works)

## TODO

- [ ] Support multiple connections per region.
- [X] Support nodes.
- [ ] Support actual nodes.
- [X] Switch to internal lock handles.
- [ ] Support all andesite features.
- [ ] Support identifiers.
- [X] Support pause, resume, skip to time.
- [X] Support starting at specific times and configurable replace current stream.
- [X] Support equalization.
- [X] Support both rustls and native_tls backends as features.
- [X] Support twilight.
- [X] Support events.
- [ ] Support raw events.
- [ ] Implement my own event handler for voice connections.
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
lavalink-rs = { version = "0.8", features = ["rustls"] }
# or
[dependencies.lavalink-rs]
version = "0.8"
features = ["rustls"]
```

Or the development release:

```toml
lavalink-rs = { git = "https://gitlab.com/vicky5124/lavalink-rs/", branch = "master" }
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

No default features are set, so you will need to specify them yourself.
These are the available ones:

- `rustls`: Use the rustls TLS backend.
- `native`: Uses the system native TLS backend (OpenSSL on linux).
- `rustls-tokio-02`: rustls, but uses tokio 0.2 instead of 1.x
- `native-tokio-02`: native, but uses tokio 0.2 instead of 1.x
- `serenity`: Add support for serenity's models.
- `twilight`: Add support for twilight-model.
- `andesite`: Add andesite exclusive features.
