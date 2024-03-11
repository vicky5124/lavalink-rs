import bot
from utils import Context, Plugin
from lavalink_voice import LavalinkVoice

import logging
import typing as t

import hikari
import lightbulb
from lavalink_rs.model.track import TrackData, PlaylistData

plugin = Plugin("Music (basic) commands")
plugin.add_checks(lightbulb.guild_only)


async def _join(ctx: Context) -> t.Optional[hikari.Snowflake]:
    if not ctx.guild_id:
        return None

    if "channel" not in ctx.options.items():
        voice_state = ctx.bot.cache.get_voice_state(ctx.guild_id, ctx.author.id)

        if not voice_state or not voice_state.channel_id:
            return None

        channel_id = voice_state.channel_id
    else:
        channel_id = ctx.options.channel.id

    voice = ctx.bot.voice.connections.get(ctx.guild_id)

    if not voice:
        voice = await LavalinkVoice.connect(
            ctx.guild_id,
            channel_id,
            ctx.bot,
            ctx.bot.lavalink,
            (ctx.channel_id, ctx.bot.rest),
        )

    return channel_id


@plugin.command()
@lightbulb.option(
    "channel",
    "The channel you want me to join",
    hikari.GuildVoiceChannel,
    required=False,
    channel_types=[hikari.ChannelType.GUILD_VOICE],
)
@lightbulb.command(
    "join", "Enters the voice channel you are connected to, or the one specified"
)
@lightbulb.implements(lightbulb.PrefixCommand, lightbulb.SlashCommand)
async def join(ctx: Context) -> None:
    """Joins the voice channel you are in"""
    channel_id = await _join(ctx)

    if channel_id:
        await ctx.respond(f"Joined <#{channel_id}>")
    else:
        await ctx.respond(
            "Please, join a voice channel, or specify a specific channel to join in"
        )


@plugin.command()
@lightbulb.command("leave", "Leaves the voice channel")
@lightbulb.implements(lightbulb.PrefixCommand, lightbulb.SlashCommand)
async def leave(ctx: Context) -> None:
    """Leaves the voice channel"""
    if not ctx.guild_id:
        return None

    voice = ctx.bot.voice.connections.get(ctx.guild_id)

    if not voice:
        await ctx.respond("Not in a voice channel")
        return None

    await voice.disconnect()

    await ctx.respond("Left the voice channel")


@plugin.command()
@lightbulb.option(
    "query",
    "The spotify search query, or any URL",
    modifier=lightbulb.OptionModifier.CONSUME_REST,
    required=False,
)
@lightbulb.command(
    "play",
    "Searches the query on spotify and adds the first result to the queue, or adds the URL to the queue",
    auto_defer=True,
)
@lightbulb.implements(
    lightbulb.PrefixCommand,
    lightbulb.SlashCommand,
)
async def play(ctx: Context) -> None:
    if not ctx.guild_id:
        return None

    voice = ctx.bot.voice.connections.get(ctx.guild_id)
    has_joined = False

    if not voice:
        if not await _join(ctx):
            await ctx.respond("Please, join a voice channel first.")
            return None
        voice = ctx.bot.voice.connections.get(ctx.guild_id)
        has_joined = True

    assert isinstance(voice, LavalinkVoice)

    player_ctx = voice.player
    query = ctx.options.query.replace(">", "").replace("<", "")

    if not query:
        player = await player_ctx.get_player()

        if not player.track and await player_ctx.get_queue():
            player_ctx.skip()
        else:
            if player.track:
                await ctx.respond("A song is already playing")
            else:
                await ctx.respond("The queue is empty")

        return None

    if not query.startswith("http"):
        query = f"spsearch:{query}"

    try:
        loaded_tracks = await ctx.bot.lavalink.load_tracks(ctx.guild_id, query)
    except Exception as e:
        logging.error(e)
        await ctx.respond("Error")
        return None

    # Single track
    if isinstance(loaded_tracks, TrackData):
        player_ctx.queue(loaded_tracks)

        if loaded_tracks.info.uri:
            await ctx.respond(
                f"Added to queue: [`{loaded_tracks.info.author} - {loaded_tracks.info.title}`](<{loaded_tracks.info.uri}>)"
            )
        else:
            await ctx.respond(
                f"Added to queue: `{loaded_tracks.info.author} - {loaded_tracks.info.title}`"
            )

    # Search results
    elif isinstance(loaded_tracks, list):
        player_ctx.queue(loaded_tracks[0])

        if loaded_tracks[0].info.uri:
            await ctx.respond(
                f"Added to queue: [`{loaded_tracks[0].info.author} - {loaded_tracks[0].info.title}`](<{loaded_tracks[0].info.uri}>)"
            )
        else:
            await ctx.respond(
                f"Added to queue: `{loaded_tracks[0].info.author} - {loaded_tracks[0].info.title}`"
            )

    # Playlist
    elif isinstance(loaded_tracks, PlaylistData):
        if loaded_tracks.info.selected_track:
            track = loaded_tracks.tracks[loaded_tracks.info.selected_track]
            player_ctx.queue(track)

            if track.info.uri:
                await ctx.respond(
                    f"Added to queue: [`{track.info.author} - {track.info.title}`](<{track.info.uri}>)"
                )
            else:
                await ctx.respond(
                    f"Added to queue: `{track.info.author} - {track.info.title}`"
                )
        else:
            player_ctx.set_queue_append(loaded_tracks.tracks)
            await ctx.respond(f"Added playlist to queue: `{loaded_tracks.info.name}`")

    # Error or no results
    else:
        await ctx.respond("No songs found")
        return None

    player_data = await player_ctx.get_player()

    if player_data:
        if not player_data.track and await player_ctx.get_queue() and not has_joined:
            player_ctx.skip()


@plugin.command()
@lightbulb.command("skip", "Skip the currently playing song")
@lightbulb.implements(lightbulb.PrefixCommand, lightbulb.SlashCommand)
async def skip(ctx: Context) -> None:
    """Skip the currently playing song"""
    if not ctx.guild_id:
        return None

    voice = ctx.bot.voice.connections.get(ctx.guild_id)

    if not voice:
        await ctx.respond("Not connected to a voice channel")
        return None

    assert isinstance(voice, LavalinkVoice)

    player = await voice.player.get_player()

    if player.track:
        if player.track.info.uri:
            await ctx.respond(
                f"Skipped: [`{player.track.info.author} - {player.track.info.title}`](<{player.track.info.uri}>)"
            )
        else:
            await ctx.respond(
                f"Skipped: `{player.track.info.author} - {player.track.info.title}`"
            )

        voice.player.skip()
    else:
        await ctx.respond("Nothing to skip")


@plugin.command()
@lightbulb.command("stop", "Stop the currently playing song")
@lightbulb.implements(lightbulb.PrefixCommand, lightbulb.SlashCommand)
async def stop(ctx: Context) -> None:
    """Stop the currently playing song"""
    if not ctx.guild_id:
        return None

    voice = ctx.bot.voice.connections.get(ctx.guild_id)

    if not voice:
        await ctx.respond("Not connected to a voice channel")
        return None

    assert isinstance(voice, LavalinkVoice)

    player = await voice.player.get_player()

    if player.track:
        if player.track.info.uri:
            await ctx.respond(
                f"Stopped: [`{player.track.info.author} - {player.track.info.title}`](<{player.track.info.uri}>)"
            )
        else:
            await ctx.respond(
                f"Stopped: `{player.track.info.author} - {player.track.info.title}`"
            )

        await voice.player.stop_now()
    else:
        await ctx.respond("Nothing to stop")


def load(bot: bot.Bot) -> None:
    bot.add_plugin(plugin)
