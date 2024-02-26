import typing as t

import hikari
from hikari import snowflakes, GatewayBot
from hikari.api import VoiceConnection, VoiceComponent

from lavalink_rs import LavalinkClient, PlayerContext


class LavalinkVoice(VoiceConnection):
    lavalink: LavalinkClient
    player: PlayerContext

    def __init__(
        self,
        player: PlayerContext,
        lavalink_client: LavalinkClient,
        *,
        channel_id: snowflakes.Snowflake,
        guild_id: snowflakes.Snowflake,
        is_alive: bool,
        shard_id: int,
        owner: VoiceComponent,
        on_close: t.Any,
    ) -> None:
        self.player = player
        self.lavalink = lavalink_client

        self.__channel_id = channel_id
        self.__guild_id = guild_id
        self.__is_alive = is_alive
        self.__shard_id = shard_id
        self.__owner = owner
        self.__on_close = on_close

    @property
    def channel_id(self) -> snowflakes.Snowflake:
        """Return the ID of the voice channel this voice connection is in."""
        return self.__channel_id

    @property
    def guild_id(self) -> snowflakes.Snowflake:
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

    async def notify(self, _event: hikari.VoiceEvent) -> None:
        """Submit an event to the voice connection to be processed."""

    @classmethod
    async def connect(
        cls,
        lavalink_client: LavalinkClient,
        client: GatewayBot,
        guild_id: snowflakes.Snowflake,
        channel_id: snowflakes.Snowflake,
    ) -> VoiceConnection:
        return await client.voice.connect_to(
            guild_id,
            channel_id,
            voice_connection_type=LavalinkVoice,
            lavalink_client=lavalink_client,
            deaf=True,
        )

    @classmethod
    async def initialize(
        cls,
        channel_id: snowflakes.Snowflake,
        endpoint: str,
        guild_id: snowflakes.Snowflake,
        on_close: t.Any,
        owner: VoiceComponent,
        session_id: str,
        shard_id: int,
        token: str,
        user_id: snowflakes.Snowflake,
        **kwargs: t.Any,
    ) -> t.Self:
        lavalink = kwargs["lavalink_client"]

        player = await lavalink.create_player_context(
            guild_id, endpoint, token, session_id
        )

        self = LavalinkVoice(
            player,
            lavalink,
            channel_id=channel_id,
            guild_id=guild_id,
            is_alive=True,
            shard_id=shard_id,
            owner=owner,
            on_close=on_close,
        )

        return self
