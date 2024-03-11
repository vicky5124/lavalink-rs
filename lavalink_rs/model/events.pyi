import typing as t
from lavalink_rs import GuildId
from lavalink_rs.model.player import State
from lavalink_rs.model.track import TrackData, TrackError

class Ready:
    session_id: str
    resumed: bool
    op: str

class PlayerUpdate:
    op: str
    guild_id: GuildId
    state: State

class Stats:
    frame_stats: t.Optional[FrameStats]
    op: str
    playing_players: int
    uptime: int
    memory: Memory
    cpu: Cpu
    players: int

class Cpu:
    cores: int
    system_load: float
    lavalink_load: float

class Memory:
    free: int
    allocated: int
    reservable: int
    used: int

class FrameStats:
    deficit: int
    nulled: int
    sent: int

class TrackStart:
    guild_id: GuildId
    track: TrackData
    op: str
    event_type: str

class TrackEnd:
    track: TrackData
    op: str
    event_type: str
    guild_id: GuildId
    reason: TrackEndReason

# TODO enum
class TrackEndReason:
    ...
    # Finished
    # LoadFailed
    # Stop
    # Replaced
    # Cleanup

class TrackException:
    op: str
    event_type: str
    track: TrackData
    guild_id: GuildId
    exception: TrackError

class TrackStuck:
    op: str
    guild_id: GuildId
    threshold_ms: int
    event_type: str
    track: TrackData

class WebSocketClosed:
    reason: str
    event_type: str
    op: str
    by_remote: bool
    code: int
    guild_id: GuildId
