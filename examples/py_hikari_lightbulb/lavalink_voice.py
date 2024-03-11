from __future__ import annotations
import typing as t

from utils import Bot

import hikari
from hikari.api import VoiceConnection, VoiceComponent
from lavalink_rs import LavalinkClient, PlayerContext
from lavalink_rs.model.http import UpdatePlayer
from lavalink_rs.model.player import ConnectionInfo


class LavalinkVoice(VoiceConnection):
    __slots__ = [
        "lavalink",
        "player",
        "__channel_id",
        "__guild_id",
        "__session_id",
        "__is_alive",
        "__shard_id",
        "__on_close",
        "__owner",
    ]
    lavalink: LavalinkClient
    player: PlayerContext

    def __init__(
        self,
        lavalink_client: LavalinkClient,
        player: PlayerContext,
        *,
        channel_id: hikari.Snowflake,
        guild_id: hikari.Snowflake,
        session_id: str,
        is_alive: bool,
        shard_id: int,
        owner: VoiceComponent,
        on_close: t.Any,
    ) -> None:
        self.player = player
        self.lavalink = lavalink_client

        self.__channel_id = channel_id
        self.__guild_id = guild_id
        self.__session_id = session_id
        self.__is_alive = is_alive
        self.__shard_id = shard_id
        self.__owner = owner
        self.__on_close = on_close

    @property
    def channel_id(self) -> hikari.Snowflake:
        """Return the ID of the voice channel this voice connection is in."""
        return self.__channel_id

    @property
    def guild_id(self) -> hikari.Snowflake:
        """Return the ID of the guild this voice connection is in."""
        return self.__guild_id

    @property
    def is_alive(self) -> bool:
        """Return `builtins.True` if the connection is alive."""
        return self.__is_alive

    @property
    def shard_id(self) -> int:
        """Return the ID of the shard that requested the connection."""
        return self.__shard_id

    @property
    def owner(self) -> VoiceComponent:
        """Return the component that is managing this connection."""
        return self.__owner

    async def disconnect(self) -> None:
        """Signal the process to shut down."""
        self.__is_alive = False
        await self.lavalink.delete_player(self.__guild_id)
        await self.__on_close(self)

    async def join(self) -> None:
        """Wait for the process to halt before continuing."""

    async def notify(self, event: hikari.VoiceEvent) -> None:
        """Submit an event to the voice connection to be processed."""
        if isinstance(event, hikari.VoiceServerUpdateEvent):
            # Handle the bot being moved frome one channel to another
            assert event.raw_endpoint
            update_player = UpdatePlayer()
            connection_info = ConnectionInfo(
                event.raw_endpoint, event.token, self.__session_id
            )
            connection_info.fix()
            update_player.voice = connection_info
            await self.player.update_player(update_player, True)

    @classmethod
    async def connect(
        cls,
        guild_id: hikari.Snowflake,
        channel_id: hikari.Snowflake,
        client: Bot,
        lavalink_client: LavalinkClient,
        player_data: t.Any,
        deaf: bool = True,
    ) -> LavalinkVoice:
        voice: LavalinkVoice = await client.voice.connect_to(
            guild_id,
            channel_id,
            voice_connection_type=LavalinkVoice,
            lavalink_client=lavalink_client,
            player_data=player_data,
            deaf=deaf,
        )

        return voice

    @classmethod
    async def initialize(
        cls,
        channel_id: hikari.Snowflake,
        endpoint: str,
        guild_id: hikari.Snowflake,
        on_close: t.Any,
        owner: VoiceComponent,
        session_id: str,
        shard_id: int,
        token: str,
        user_id: hikari.Snowflake,
        **kwargs: t.Any,
    ) -> LavalinkVoice:
        del user_id
        lavalink_client = kwargs["lavalink_client"]

        player = await lavalink_client.create_player_context(
            guild_id, endpoint, token, session_id
        )

        player_data = kwargs["player_data"]

        if player_data:
            player.data = player_data

        self = LavalinkVoice(
            lavalink_client,
            player,
            channel_id=channel_id,
            guild_id=guild_id,
            session_id=session_id,
            is_alive=True,
            shard_id=shard_id,
            owner=owner,
            on_close=on_close,
        )

        return self
