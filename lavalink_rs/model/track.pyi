import typing as t

# TODO enum
class TrackLoadType: ...

class TrackData:
    info: TrackInfo
    encoded: str

class TrackInfo:
    identifier: str
    source_name: str
    is_seekable: bool
    title: str
    is_stream: bool
    isrc: t.Optional[str]
    artwork_url: t.Optional[str]
    author: str
    position: int
    uri: t.Optional[str]
    length: int

class PlaylistData:
    tracks: t.List[TrackData]
    info: PlaylistInfo

class PlaylistInfo:
    name: str
    selected_track: t.Optional[int]

class TrackError:
    message: str
    severity: str
    cause: str
