import os
import logging
import typing as t

from lavalink_voice import LavalinkVoice

import hikari

import lavalink_rs
from lavalink_rs import LavalinkClient, NodeDistributionStrategy
from lavalink_rs.model.track import TrackData, PlaylistData
from lavalink_rs.model import events  # , GuildId


class Data:
    """Global data shared across the entire bot, used to store dashboard values."""

    def __init__(self) -> None:
        self.lavalink: LavalinkClient


class Bot(hikari.GatewayBot):
    """Just implementing the data to the Bot."""

    def __init__(self, **kwargs: t.Any) -> None:
        super().__init__(**kwargs)
        self.data = Data()


class Events:
    async def ready(
        self, client: LavalinkClient, session_id: str, event: events.Ready
    ) -> None:
        logging.info("HOLY READY")

    async def track_start(
        self,
        client: LavalinkClient,
        session_id: str,
        event: events.TrackStart,
    ) -> None:
        logging.info(
            f"Started track {event.track.info.author} - {event.track.info.title} in {event.guild_id.inner}"
        )

        player_ctx = client.get_player_context(event.guild_id.inner)
        assert player_ctx
        assert player_ctx.data

        await bot.rest.create_message(
            player_ctx.data,
            f"Started playing `{event.track.info.author} - {event.track.info.title}`",
        )


bot = Bot(token=os.environ["DISCORD_TOKEN"], intents=hikari.intents.Intents.ALL)

logging.getLogger().setLevel(logging.DEBUG)


# You can make your own node selection algorythm, and return the index of that node.
# The index is the same as when they were added to the client initially.
# async def custom(client: LavalinkClient, guild_id: GuildId) -> int:
#    return 0


@bot.listen()
async def on_starting(_event: hikari.StartingEvent) -> None:
    node = lavalink_rs.NodeBuilder(
        "localhost:2333",
        False,  # is the server SSL?
        os.environ["LAVALINK_PASSWORD"],
        601749512456896522,  # Bot ID
    )

    lavalink_events = t.cast(lavalink_rs.EventHandler, Events())

    lavalink_client = await LavalinkClient.new(
        lavalink_events,
        [node],
        # NodeDistributionStrategy.custom(custom),
        NodeDistributionStrategy.sharded(),
        # 123 is any python object, "123" is used as an exaple in user data with `,test`.
        123,
    )
    bot.data.lavalink = lavalink_client


@bot.listen()
async def on_message(event: hikari.GuildMessageCreateEvent) -> None:
    # Do not respond to bots nor webhooks pinging us, only user accounts
    if not event.is_human or not event.message.content:
        return None

    if event.message.content.startswith(",test"):
        voice = bot.voice.connections.get(event.guild_id)
        assert isinstance(voice, LavalinkVoice)

        if not voice:
            await event.message.respond("Not in a voice channel.")
            return None

        # Custom user data can be accessessed via the data getter and setter of the client.
        assert voice.lavalink.data
        voice.lavalink.data += 1
        await event.message.respond(f"Test data {voice.lavalink.data}")

    elif event.message.content.startswith(",leave"):
        voice = bot.voice.connections.get(event.guild_id)
        assert isinstance(voice, LavalinkVoice)

        if not voice:
            await event.message.respond("Not in a voice channel.")
            return None

        await voice.lavalink.delete_player(event.guild_id)
        await voice.disconnect()

        await event.message.respond("Left voice channel.")

    elif event.message.content.startswith(",play"):
        if not (voice_state := bot.cache.get_voice_state(event.guild_id, event.author)):
            await event.message.respond("Connect to a voice channel first")
            return None

        channel_id = voice_state.channel_id

        assert channel_id is not None

        voice = bot.voice.connections.get(event.guild_id)

        if not voice:
            voice = await LavalinkVoice.connect(
                bot.data.lavalink, bot, event.guild_id, channel_id
            )
            await event.message.respond(f"Joined <#{channel_id}>")

        assert isinstance(voice, LavalinkVoice)

        player_ctx = voice.player

        # Just like with the client, you can add custom data to the player context either
        # with an argument in `create_player_context` or using the data setter.
        player_ctx.data = event.message.channel_id

        query = event.message.content.replace(",play ", "")

        if not query.startswith("http"):
            query = f"ytsearch:{query}"

        if not query:
            await event.message.respond("Nothing to play.")
            return None

        loaded_tracks = await bot.data.lavalink.load_tracks(event.guild_id, query)

        # Single track
        if isinstance(loaded_tracks, TrackData):
            player_ctx.queue(loaded_tracks)
            await event.message.respond(
                f"Added to queue: {loaded_tracks.info.author} - {loaded_tracks.info.title}"
            )

        # Search results
        elif isinstance(loaded_tracks, list):
            player_ctx.queue(loaded_tracks[0])
            await event.message.respond(
                f"Added to queue: {loaded_tracks[0].info.author} - {loaded_tracks[0].info.title}"
            )

        # Playlist
        elif isinstance(loaded_tracks, PlaylistData):
            player_ctx.set_queue_append(loaded_tracks.tracks)
            await event.message.respond(
                f"Added playlist to queue: {loaded_tracks.info.name}"
            )

        # Error or no results
        else:
            logging.error(loaded_tracks)
            await event.message.respond("Error.")
            return None

        player_data = await player_ctx.get_player()

        if player_data:
            if not player_data.track and await player_ctx.get_queue():
                player_ctx.skip()


bot.run()
