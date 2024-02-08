# lavalink-rs

An API Wrapper for `lavalink`. Compatible with all `tokio 1.x` based discord crates or `asyncio` based discord python libraries.

## To-Do

- [ ] Streamable queue reader
- [ ] Improve documentation with examples, better formatting, and fill in missing data
- [ ] Expand event logging
- [ ] Examples
- [ ] Gitlab CI Tests
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
- [x] Lavasnek (PyO3)
- [x] Lavasnek events
- [x] Remove third party dependency for custom user data.

## Links to download stuff you will need

To install Lavalink, you can follow their [getting started guide](https://lavalink.dev/getting-started.html).

## How to use

Install the version from crates.io:

```toml
lavalink-rs = "0.10"

# or

[dependencies.lavalink-rs]
version = "0.10.0"
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

To build for python, you can use maturin.

```
python3 -m venv .venv
source .venv/bin/activate
python3 -m pip install maturin
maturin develop --target x86_64-unknown-linux-gnu
```

### Features

- `rustls`: **default feature** - Use rustls.
- `native-tls` Use the system native tls.
- `serenity` for [serenity](https://lib.rs/crates/serenity) support.
- `songbird` for [songbird](https://lib.rs/crates/songbird) support.
- `twilight` for [twilight-model](https://lib.rs/crates/twilight-model) support.
- `python` for python3.8+ support.
