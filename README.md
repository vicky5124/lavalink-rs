# Serenity Lavalink
Note: This is not a forl of the [official repository](https://github.com/serenity-rs/serenity-lavalink) with the same name, this is a completely new wrapper from scratch.

### Links you will need
- [Lavalink repository](https://github.com/Frederikam/Lavalink)
- [Java download](https://jdk.java.net/archive/) (11 or newer, 13 recommended)

### TODO
- [ ] Support multiple connections per region.
- [X] Support nodes.
- [X] Support pause, resume, skip to time.
- [X] Support starting at specific times and configurable replace current stream.
- [ ] Support equalization.
- [ ] Support identifiers.
- [ ] Support both rustls and native_tls backends as features.
- [ ] Support twilight.
- [ ] Support raw events.
- [ ] Implement my own event hander
- [ ] Implement my own websocket connection to voice channels.
- [X] Support easy queues natively.
- [ ] Optimize the codebase.
- [ ] Remove all the clones from examples.
- [ ] Improve error handling.
- [ ] Add tracing and logging.
- [ ] Add documentation.

### How to use
```toml
serenity_lavalink = { git = "https://gitlab.com/nitsuga5124/serenity-lavalink/", branch = "master" }
```
