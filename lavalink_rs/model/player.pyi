import typing as t

from lavalink_rs import GuildId
from lavalink_rs.model.track import TrackData

class Player:
    track: t.Optional[TrackData]
    volume: int
    voice: ConnectionInfo
    guild_id: GuildId
    paused: bool
    state: State
    filters: t.Optional[Filters]

class State:
    connected: bool
    time: int
    position: int
    ping: t.Optional[int]

class ConnectionInfo:
    session_id: str
    token: str
    endpoint: str

    def __new__(cls, endpoint: str, token: str, session_id: str) -> ConnectionInfo: ...
    def fix(self) -> None: ...

class Filters:
    tremolo: t.Optional[TremoloVibrato]
    low_pass: t.Optional[LowPass]
    distortion: t.Optional[Distortion]
    rotation: t.Optional[Rotation]
    karaoke: t.Optional[Karaoke]
    equalizer: t.Optional[t.List[Equalizer]]
    volume: t.Optional[int]
    vibrato: t.Optional[TremoloVibrato]
    timescale: t.Optional[Timescale]
    channel_mix: t.Optional[ChannelMix]

    def __new__(cls) -> Filters: ...

class ChannelMix:
    right_to_left: t.Optional[float]
    right_to_right: t.Optional[float]
    left_to_right: t.Optional[float]
    left_to_left: t.Optional[float]

class Distortion:
    sin_offset: t.Optional[float]
    scale: t.Optional[float]
    offset: t.Optional[float]
    sin_scale: t.Optional[float]
    cos_offset: t.Optional[float]
    tan_offset: t.Optional[float]
    cos_scale: t.Optional[float]
    tan_scale: t.Optional[float]

    def __new__(cls) -> Distortion: ...

class Equalizer:
    gain: float
    band: int

    def __new__(cls) -> Equalizer: ...

class Karaoke:
    filter_band: t.Optional[float]
    level: t.Optional[float]
    filter_width: t.Optional[float]
    mono_level: t.Optional[float]

    def __new__(cls) -> Karaoke: ...

class LowPass:
    smoothing: t.Optional[float]

    def __new__(cls) -> LowPass: ...

class Rotation:
    rotation_hz: t.Optional[float]

    def __new__(cls) -> Rotation: ...

class Timescale:
    speed: t.Optional[float]
    pitch: t.Optional[float]
    rate: t.Optional[float]

    def __new__(cls) -> Timescale: ...

class TremoloVibrato:
    frequency: t.Optional[float]
    depth: t.Optional[float]

    def __new__(cls) -> TremoloVibrato: ...
