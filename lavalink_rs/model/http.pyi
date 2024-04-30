import typing as t

from lavalink_rs.model.player import Filters, ConnectionInfo

JSON: t.TypeAlias = t.Union[
    t.Dict[str, "JSON"], t.List["JSON"], str, int, float, bool, None
]

class UpdatePlayer:
    end_time: t.Optional[int]
    volume: t.Optional[int]
    position: t.Optional[int]
    filters: t.Optional[Filters]
    voice: t.Optional[ConnectionInfo]
    paused: t.Optional[bool]
    track: t.Optional[UpdatePlayerTrack]
    user_data: t.Optional[JSON]

class UpdatePlayerTrack:
    encoded: t.Optional[str]
    identifier: t.Optional[str]

class ResumingState:
    timeout: t.Optional[int]
    resuming: t.Optional[bool]

class Info:
    build_time: int
    plugins: t.List[Plugin]
    git: Git
    filters: t.List[str]
    version: Version
    source_managers: t.List[str]
    jvm: str
    lavaplayer: str

class Git:
    commit: str
    commit_time: int
    branch: str

class Plugin:
    version: str
    name: str

class Version:
    pre_release: t.Optional[str]
    major: int
    patch: int
    semver: str
    minor: int
    build: t.Optional[str]
