import os
import logging
import typing as t

from lavalink_voice import LavalinkVoice

import hikari
import lavalink_rs
from lavalink_rs.model.track import TrackData, PlaylistData
from lavalink_rs.model import events


class Data:
    """Global data shared across the entire bot, used to store dashboard values."""

    def __init__(self) -> None:
        self.lavalink: lavalink_rs.Lavalink


class Bot(hikari.GatewayBot):
    """Just implementing the data to the Bot."""

    def __init__(self, **kwargs: t.Any) -> None:
        super().__init__(**kwargs)
        self.data = Data()


class Events:
    async def ready(self, client: lavalink_rs.LavalinkClient, session_id: str, event: events.Ready):
        logging.info("HOLY READY")

    async def track_start(self, client: lavalink_rs.LavalinkClient, session_id: str, event: events.TrackStart):
        logging.info(f"Started track {event.track.info.author} - {event.track.info.title} in {event.guild_id.inner}")


bot = Bot(token=os.environ["DISCORD_TOKEN"], intents=hikari.intents.Intents.ALL)

logging.getLogger().setLevel(logging.DEBUG)


@bot.listen()
async def on_starting(_event: hikari.StartingEvent) -> None:
    node = lavalink_rs.NodeBuilder(
        "localhost:2333",
        False, # is the server SSL?
        os.environ["LAVALINK_PASSWORD"],
        601749512456896522, # Bot ID
    )
    lavalink_client = lavalink_rs.LavalinkClient([node], Events())
    await lavalink_client.start()
    bot.data.lavalink_client = lavalink_client


@bot.listen()
async def on_message(event: hikari.GuildMessageCreateEvent) -> None:
    # Do not respond to bots nor webhooks pinging us, only user accounts
    if not event.is_human or not event.message.content:
        return None

    if event.message.content.startswith(",test"):
        await event.message.respond("AAAAAAAAAAAAAAA")

    elif event.message.content.startswith(",leave"):
        voice = bot.voice.connections.get(event.guild_id)

        if not voice:
            await event.message.respond(f"Not in a voice channel.")
            return None

        await voice.lavalink_client.delete_player(event.guild_id)
        await voice.disconnect()

        await event.message.respond(f"Left voice channel.")

    elif event.message.content.startswith(",play"):
        if not (voice_state := bot.cache.get_voice_state(event.guild_id, event.author)):
            await event.message.respond("Connect to a voice channel first")
            return None

        channel_id = voice_state.channel_id

        assert channel_id is not None

        voice = bot.voice.connections.get(event.guild_id)

        if not voice:
            voice = await LavalinkVoice.connect(
                bot.data.lavalink_client, bot, event.guild_id, channel_id
            )
            await event.message.respond(f"Joined <#{channel_id}>")

        player_ctx = voice.player

        query = event.message.content.replace(",play ", "")

        if not query.startswith("http"):
            query = f"ytsearch:{query}"

        if not query:
            await event.message.respond("Nothing to play.")
            return None

        loaded_tracks = await bot.data.lavalink_client.load_tracks(
            event.guild_id, query
        )

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
