import typing as t

from lavalink_rs.model.player import ConnectionInfo, Player, Filters
from lavalink_rs.model.http import UpdatePlayer, Info
from lavalink_rs.model.track import TrackData, PlaylistData, TrackError
from lavalink_rs.model.events import (
    Stats,
    PlayerUpdate,
    TrackStart,
    TrackEnd,
    TrackException,
    TrackStuck,
    WebSocketClosed,
    Ready,
)

__CD = t.TypeVar("__CD")
__PD = t.TypeVar("__PD")

class LavalinkClient:
    data: t.Optional[__CD]

    @staticmethod
    async def new(
        events: EventHandler,
        nodes: t.List[NodeBuilder],
        strategy: NodeDistributionStrategy,
        data: t.Optional[__CD] = None,
    ) -> LavalinkClient: ...
    def get_player_context(
        self, guild_id: t.Union[GuildId, int]
    ) -> t.Optional[PlayerContext]: ...
    async def create_player(
        self, guild_id: t.Union[GuildId, int], connection_info: ConnectionInfo
    ) -> Player: ...
    async def create_player_context(
        self,
        guild_id: t.Union[GuildId, int],
        endpoint: str,
        token: str,
        session_id: str,
        data: t.Optional[__PD] = None,
    ) -> PlayerContext: ...
    async def delete_player(self, guild_id: t.Union[GuildId, int]) -> None: ...
    async def delete_all_player_contexts(self) -> None: ...
    async def update_player(
        self,
        guild_id: t.Union[GuildId, int],
        update_player: UpdatePlayer,
        no_replace: bool,
    ) -> Player: ...
    async def load_tracks(
        self, guild_id: t.Union[GuildId, int], identifier: str
    ) -> t.Optional[
        t.Union[TrackData, PlaylistData, t.List[TrackData], TrackError]
    ]: ...
    async def decode_track(
        self, guild_id: t.Union[GuildId, int], track: str
    ) -> TrackData: ...
    async def decode_tracks(
        self, guild_id: t.Union[GuildId, int], tracks: t.List[str]
    ) -> t.List[TrackData]: ...
    async def request_version(self, guild_id: t.Union[GuildId, int]) -> str: ...
    async def request_stats(self, guild_id: t.Union[GuildId, int]) -> Stats: ...
    async def request_info(self, guild_id: t.Union[GuildId, int]) -> Info: ...
    async def request_player(self, guild_id: t.Union[GuildId, int]) -> Player: ...
    async def request_all_players(
        self, guild_id: t.Union[GuildId, int]
    ) -> t.List[Player]: ...
    def handle_voice_server_update(
        self, guild_id: t.Union[GuildId, int], token: str, endpoint: t.Optional[str]
    ) -> None: ...
    def handle_voice_state_update(
        self,
        guild_id: t.Union[GuildId, int],
        channel_id: t.Optional[t.Union[ChannelId, int]],
        user_id: t.Union[UserId, int],
        session_id: str,
    ) -> None: ...
    async def get_connection_info(
        self, guild_id: t.Union[GuildId, int], timeout: int
    ) -> ConnectionInfo: ...

class PlayerContext:
    data: t.Optional[__PD]

    def close(self) -> None: ...
    def skip(self) -> None: ...
    def finish(self, should_continue: bool) -> None: ...
    def update_player_data(self, player: Player) -> None: ...
    async def get_player(self) -> Player: ...
    async def update_player(
        self, update_player: UpdatePlayer, no_replace: bool
    ) -> Player: ...
    async def play(self, track: TrackData) -> Player: ...
    async def play_now(self, track: TrackData) -> Player: ...
    async def stop_now(self) -> Player: ...
    async def set_pause(self, pause: bool) -> Player: ...
    async def set_volume(self, volume: int) -> Player: ...
    async def set_filters(self, filters: Filters) -> Player: ...
    async def set_position_ms(self, position: int) -> Player: ...
    def queue(self, track: t.Union[TrackInQueue, TrackData]) -> None: ...
    async def get_queue(self) -> t.List[TrackInQueue]: ...
    def set_queue_push_to_back(
        self, track: t.Union[TrackInQueue, TrackData]
    ) -> None: ...
    def set_queue_push_to_front(
        self, track: t.Union[TrackInQueue, TrackData]
    ) -> None: ...
    def set_queue_insert(
        self, position: int, track: t.Union[TrackInQueue, TrackData]
    ) -> None: ...
    def set_queue_remove(self, position: int) -> None: ...
    def set_queue_clear(self) -> None: ...
    def set_queue_replace(
        self, tracks: t.Sequence[t.Union[TrackInQueue, TrackData]]
    ) -> None: ...
    def set_queue_append(
        self, tracks: t.Sequence[t.Union[TrackInQueue, TrackData]]
    ) -> None: ...

class NodeBuilder:
    hostname: str
    is_ssl: bool
    password: str
    user_id: UserId
    session_id: t.Optional[str]

    def __init__(
        self,
        hostname: str,
        is_ssl: bool,
        password: str,
        user_id: t.Union[UserId, int],
        session_id: t.Optional[str] = None,
        events: t.Optional[EventHandler] = None,
    ) -> None: ...

class EventHandler:
    async def stats(
        self, client: LavalinkClient, session_id: str, event: Stats
    ) -> None: ...
    async def player_update(
        self, client: LavalinkClient, session_id: str, event: PlayerUpdate
    ) -> None: ...
    async def track_start(
        self, client: LavalinkClient, session_id: str, event: TrackStart
    ) -> None: ...
    async def track_end(
        self, client: LavalinkClient, session_id: str, event: TrackEnd
    ) -> None: ...
    async def track_exception(
        self, client: LavalinkClient, session_id: str, event: TrackException
    ) -> None: ...
    async def track_stuck(
        self, client: LavalinkClient, session_id: str, event: TrackStuck
    ) -> None: ...
    async def websocket_closed(
        self, client: LavalinkClient, session_id: str, event: WebSocketClosed
    ) -> None: ...
    async def ready(
        self, client: LavalinkClient, session_id: str, event: Ready
    ) -> None: ...

class NodeDistributionStrategy:
    @staticmethod
    def new() -> NodeDistributionStrategy: ...
    @staticmethod
    def sharded() -> NodeDistributionStrategy: ...
    @staticmethod
    def round_robin() -> NodeDistributionStrategy: ...
    @staticmethod
    def main_fallback() -> NodeDistributionStrategy: ...
    @staticmethod
    def lowest_load() -> NodeDistributionStrategy: ...
    @staticmethod
    def highest_free_memory() -> NodeDistributionStrategy: ...
    @staticmethod
    def custom(
        func: t.Callable[[LavalinkClient, t.Union[GuildId, int]], t.Awaitable[int]],
    ) -> NodeDistributionStrategy: ...

class TrackInQueue:
    track: TrackData
    volume: t.Optional[int]
    end_time_ms: t.Optional[int]
    start_time_ms: t.Optional[int]
    filters: t.Optional[Filters]

class GuildId:
    inner: int

    def __init__(self, id: int) -> None: ...

class ChannelId:
    inner: int

    def __init__(self, id: int) -> None: ...

class UserId:
    inner: int

    def __init__(self, id: int) -> None: ...
