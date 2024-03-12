# lavalink-rs

An API Wrapper for `lavalink`. Compatible with all `tokio 1.x` based discord crates or `asyncio` based discord python libraries.

If you have questions, you can get support in the [serenity](https://discord.gg/serenity-rs), [lavalink](https://discord.gg/2rpnXNfRRU) or [hikari](https://discord.gg/hikari)
discord servers, or by opening an issue in the [gitlab repository](https://gitlab.com/vicky5124/lavalink-rs).

Documentation for the library can be found [here for rust](https://docs.rs/lavalink-rs) or [here for python](https://vicky5124.github.io/lavalink-rs/lavalink_rs.html).

## To-Do

### 0.11.2

- [x] hikari-lightbulb example
- [x] Implement python builders for all the needed model structures

### 0.11.3

- [ ] Expose Http and methods to python
- [ ] Implement search utilities to python

### 0.12

- [ ] Implement some const methods
- [ ] RoutePlanner API
- [ ] Switch from reqwests to hyper
- [ ] native and webpki roots for rustls feature separation
- [ ] Support `tokio-websockets`

### Future

- [ ] Streamable queue reader
- [ ] Improve documentation with examples, better formatting, and fill in missing data
- [ ] Expand event logging
- [ ] discord.py example
- [ ] hata example
- [ ] twilight-rs example
- [ ] Gitlab CI Tests
- [ ] Implement abstractions for ease of use
- [ ] Region based node selection method

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
- [x] Round-Robin node selection method
- [x] Main and fallback node selection method
- [x] CPU Load based node selection method
- [x] Memory usage based node selection method
- [x] Custom node selection method
- [x] Python stubs
- [x] Basic Twilight 0.16 support
- [x] Hide password from logs

## Links to download stuff you will need

To install Lavalink, you can follow their [getting started guide](https://lavalink.dev/getting-started/index.html).

## How to use

This is how you can install the library to be used.

### Rust

You can install the latest version from crates.io:

```toml
lavalink-rs = "0.11"

# or

[dependencies.lavalink-rs]
version = "0.11"
```

Or the development release:

```toml
lavalink-rs = { git = "https://gitlab.com/vicky5124/lavalink-rs/", branch = "main"}

# or

[dependencies.lavalink-rs]
git = "https://gitlab.com/vicky5124/lavalink-rs/"
branch = "main"
```

If you wish to use a development version of songbird (or serenity, or twilight-model), add the following to the Cargo.toml:

```toml
[patch.crates-io.songbird]
git = "https://github.com/serenity-rs/songbird"
branch = "next"

[dependencies.songbird]
git = "https://github.com/serenity-rs/songbird"
branch = "next"
```

### Python

You can install the latest version from [pypi](https://pypi.org/project/lavalink_rs/) by using `python3 -m pip install lavalink_rs`.

OS | Arch | Python Versions | Supported?
-|-|-|-
Windows | x86 | 3.8-3.12 | ✅
Windows | x64 | 3.8-3.12 | ✅
Windows | aarch64 | - | ❌
Mac OS | x86_64 | 3.8-3.12 | ✅
Mac OS | aarch64 | 3.8-3.12 | ✅
Linux | x86 | 3.8-3.12 | ✅
Linux | x86_64 | 3.8-3.12 | ✅
Linux | armv7 | 3.8-3.12 | ✅
Linux | aarch64 | 3.8-3.12 | ✅
Linux | ppc64le | 3.8-3.12 | ✅
Linux | s390x | 3.8-3.12 | ✅

## Building the library

You can build the library using `cargo build`

To build for python, you can use maturin instead.

```
python3 -m venv .venv
source .venv/bin/activate
python3 -m pip install maturin
maturin develop --target x86_64-unknown-linux-gnu
```

## Features

- `macros`: **default feature** - Adds procedural macros for ease of use.
- `rustls`: **default feature** - Use rustls.
- `native-tls` Use the system native tls.
- `serenity` for [serenity](https://lib.rs/crates/serenity) support.
- `songbird` for [songbird](https://lib.rs/crates/songbird) support.
- `twilight` for [twilight-model](https://lib.rs/crates/twilight-model) v0.15 support.
- `twilight16` for [twilight-model](https://lib.rs/crates/twilight-model) v0.16-rc support.
- `python` for python3.8+ support.

## Contributing

To contribute to the project, fork the [gitlab repository](https://gitlab.com/vicky5124/lavalink-rs) and create a merge request over there. Make sure to update the changelog with whatever update you did to the library.
