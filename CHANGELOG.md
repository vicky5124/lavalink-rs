# Changelog

## 0.11.2

- Implement a hikari-lightbulb example.
- Implement constructors for the structures that should be contructable within python.
- Change code formatting to ruff instead of black.
- Add @staticmethod flags to the stubs.
- Become pyright complient.
- Remove the pure hikari example

## 0.11.1

- Fix python version limit upper bound to include every 3.12 version.

## 0.11.0

- Remove the typemap_rev dependency.
- Remove user-data feature.
- Move user-data to a standard rust implementation inspired by 0.13 serenity.
- Merged `serenity-native` and `serenity-rustls` into a single `serenity` feature.
- Implement `PlayerContext::play()`
- Switched from `async-tungstenite` to `tokio-tungstenite`
- Move the hook macro inside the library.
- Implement user data in lavasnek.
- Implement discord voice event handling and `ConnectionInfo` creation utilities.
- Implement python stubs.
- Create python documentation.
- Support twilight-model 0.16
- Hide passwords from logs.
- Implemented node selection methods:
  - Round-Robin.
  - Main and fallback.
  - Lowest CPU load.
  - Most memory free.
  - Custom method.

## 0.10.0

- Implement events in python.
- Update dependencies.

## 0.10.0-beta.3

- Initial Python implementation.

## 0.10.0-beta.2

- Implement additional queue altering actions.
- Fix stop request.
- Fix skip on an empty queue.
- Implement raw REST requests.
- Expand the poise example functionality.

## 0.10.0-beta.1

- Replace all u128 with u64 due to [serde issue](https://github.com/serde-rs/json/issues/625).

## 0.10.0-beta.0

- Implement native-tls support.
- Improve user data.
- Document code.

## 0.10.0-alpha.3

- Implemented search engine helpers with plugin support.
- Implemented a fix for player context death on lavalink restart.
- Add support for songbird.
- Add support for serenity.
- Add support for twilight-model.
- Switch reqwest from native-tls to rustls.

## 0.10.0-alpha.2

- Implement readers for queues and players.
- Remove main.rs and move to its own example.

## 0.10.0-alpha.1

- Implement write-only queues and players.

## 0.10.0-alpha.0

- Complete rewrite of the library.

## 0.9.0-rc.3

- Replace all tokio locks with parking_lot.
- Create session on voice_server_update.
- create_session() no longer creates a node if there's an existing one.
- Added track_exception and track_stuck events.
- Implement Clone for the track exception event.
- Fully reconnect if session became invalid.
- Pause and resume after creating a session on voice server update.
- Fix log levels.
- Add a way to be able to generate a TrackQueue from the PlayParameters builder.
- Fixed Panic on TrackEndEvent.
- Try to stop holding dead connections.
- Renamed simple-gateway feature with discord-gateway.
- Actually toggle is_paused on the node if available.
- Add a way to decode information from a Track BASE64.
- Added the Client-Name header that Lavalink asks for. (@zedseven)
- Implement lavalink reconnecting.

## 0.9.0-rc.2

- typemap_rev is now re-exported.
- Automatically remove Removed wss:// from endpoint if present.
- Add functions to wait for ConnectionInfo insert and delete.
- Implemented public raw event handlers.
- Make the wait time before connecting customizable.
- Allow to configure the discord gateway auto-start.
- Fix reconnect unwrap if the previous reconnect worked.
- Some structs now support Serde.
- Check for both Server and State events in join()
- Remove andesite support due to it getting archived.
- Switched serenity example back to songbird.
- Builders now take and return &mut.
- Feature gate tracing, and add log as a possible logger.
- Updated async_tungstenite.
- Switched log levels in some messages.

## 0.9.0-rc.1

- Feature gated songbird.
- Added a simple voice connection handler.
- Removed tokio 0.2 support.

## 0.9.0-rc.0

- Updated Songbird.
- Updated Twilight.
- Updated Tungstenite.
- Updated readme to include missing required features.

## 0.8.0

- Fixed typo on get_tracks()
- Transitioned to Vicky
- Create the Node on create_session() rather than queue()
- Fixed warning on twilight example

## 0.7.2

- Add equalize_dynamic() method

## 0.7.1

- Added a minimum rust version check.

## 0.7.0

- Added documentation.
- Added examples to the documentation.
- Removed unused type alias.
- Removed unused Error variants.

## 0.7.0-rc.0

- Added twilight support
- Added serenity and twilight features.
- Added 2 andesite (exclusive?) events.
- Moved away from --example builds.
- Added twilight example.

## 0.6.0

- Added tracing and event logging.

## 0.6.0-rc.1

- Exposed loops() field via method.
- Fixed self on play in the example.

## 0.6.0-rc.0

- YaY another rewrite! *mostly*
- Switched to internal locks only.
- Added builder module.
- Added a LavalinkClient builder.
- Removed unneeded function parameters.
- Removed unneeded `Option<T>`.
- Removed unneeded Errors.
- Implement `From<Error>` for various external library errors.
- Made `LavalinkClient.set_addr()` take an `impl Into<_>`.

## 0.5.4

- Added LavalinkClient.set_addr() - @suisei #8
- Fixed deserialization error causing a bad result on track loading.

## 0.5.3

- Added basic andesite feature to fix the different playlist information sent by it

## 0.5.2

- Properly handle no features
- Fixed docs.rs build

## 0.5.1

- Fixed reqwest tls features
- Added missing features warning

## 0.5.0

- First non-alpha release
- Added support for native-tls and rustls as features
- Added support for tokio 0.2 as features

## 0.4.0-alpha

- Switched to Songbird
- Updated to tokio 1.0
- Updated to serenity 0.10

## 0.3.0-alpha

- Updated serenity to 0.9

## 0.2.2-alpha

- Fixed deadlock
- Queue loop can now be closed

## 0.2.1-alpha

- Fixed dev branches of lavalink
- Removed mutable references on the play builder

## 0.2.0-alpha

- (Breaking) added data field to Node.
- Update now_playing position in player_update
- Add requester field to TrackQueue
- Destroy now skips if possible.

## 0.1.4-alpha

- UserId parameters take Into trait.
- Added some methods to GuildId.

## 0.1.3-alpha

- Added has to GuildID

## 0.1.2-alpha

- added equalization support.

## 0.1.1-alpha

- Pushed to crates.io

## 0.1.0-alpha

- Rewrote the library.
- Added events.
- Optimized the codebase.
- Removed all the clones from the examples.
- Remade easy queues.

## 0.0.3-alpha

- Added easy queues
- Added nodes

## 0.0.2-alpha

- Added start time to play()
- Added finish time to play()
- Added overwrite current stream to play()
- Added pause()
- Added resume()
- Added stop()
- Added destroy()
- Added seek()
- Added set_volume()
- Updated serenity.

## 0.0.1-alpha

- Initial release
