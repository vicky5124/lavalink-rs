# Changelog

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
