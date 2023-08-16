# lavalink-rs

An API Wrapper for `lavalink`. Compatible with all `tokio 1.x` based discord crates.

## To-Do

- [ ] Streamable queue reader
- [ ] Improve documentation with examples, better formatting, and fill in missing data
- [ ] Expand event logging
- [ ] Examples
- [ ] Gitlab CI Tests
- [ ] Lavasnek (PyO3)
- [ ] Implement abstractions for ease of use
- [ ] Round-Robin node selection method
- [ ] Region based node selection method
- [ ] Main and fallback node selection method

### Done

- [x] `native_tls` backend
- [x] Player queues
- [x] Readbale player queues
- [x] Search engine helpers
- [x] Write basic cocumentation
- [x] Songbird utilities
- [x] Serenity utilities
- [x] Twilight utilities
- [x] Load balancer node selection method

## Links to download stuff you will need

To install Lavalink, you can follow their [getting started guide](https://lavalink.dev/getting-started.html).

## How to use

Install the version from crates.io:

```toml
lavalink-rs = "0.10.0-beta"

# or

[dependencies.lavalink-rs]
version = "0.10.0-beta"
```

Or the development release:

```toml
lavalink-rs = { git = "https://gitlab.com/vicky5124/lavalink-rs/", branch = "main"}

# or

[dependencies.lavalink-rs]
git = "https://gitlab.com/vicky5124/lavalink-rs/"
branch = "main"
```

If you wish to use a development version of songbird, add the following to the Cargo.toml:

```toml
[patch.crates-io.serenity]
git = "https://github.com/serenity-rs/songbird"
branch = "next"
```

### Features

- `user-data` - **default feature** - Allows the client and player context to store custom user data.
- `rustls`: **default feature** - Use rustls.
- `native-tls` Use the system native tls.
- `serenity-rustls` for [serenity](https://lib.rs/crates/serenity) with rustls support.
- `serenity-native` for [serenity](https://lib.rs/crates/serenity) with native-tls support.
- `songbird` for [songbird](https://lib.rs/crates/songbird) support.
- `twilight` for [twilight-model](https://lib.rs/crates/twilight-model) support.
