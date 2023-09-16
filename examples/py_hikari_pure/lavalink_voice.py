import hikari
from hikari import snowflakes, GatewayBot
from hikari.api import VoiceConnection, VoiceComponent


class LavalinkVoice(VoiceConnection):
    def __init__(
        self,
        player,
        lavalink_client,
        *,
        channel_id: snowflakes.Snowflake,
        guild_id: snowflakes.Snowflake,
        is_alive: bool,
        shard_id: int,
        owner: VoiceComponent,
        on_close,
    ) -> None:
        self.player = player
        self.lavalink_client = lavalink_client

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
        # await self.player.()
        await self.__on_close(self)

    async def join(self) -> None:
        """Wait for the process to halt before continuing."""

    async def notify(self, _event: hikari.VoiceEvent) -> None:
        """Submit an event to the voice connection to be processed."""

    @classmethod
    async def connect(
        cls,
        lavalink_client,
        client: GatewayBot,
        guild_id: snowflakes.Snowflake,
        channel_id: snowflakes.Snowflake,
    ):
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
        on_close,
        owner: VoiceComponent,
        session_id: str,
        shard_id: int,
        token: str,
        user_id: snowflakes.Snowflake,
        **kwargs,
    ):
        lavalink_client = kwargs["lavalink_client"]

        player = await lavalink_client.create_player_context(
            guild_id, endpoint, token, session_id
        )

        self = LavalinkVoice(
            player,
            lavalink_client,
            channel_id=channel_id,
            guild_id=guild_id,
            is_alive=True,
            shard_id=shard_id,
            owner=owner,
            on_close=on_close,
        )

        return self
