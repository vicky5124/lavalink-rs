import typing as t

class TrackLoadType:
    Track: TrackLoadType
    Playlist: TrackLoadType
    Search: TrackLoadType
    Empty: TrackLoadType
    Error: TrackLoadType

class Track:
    load_type: TrackLoadType
    data: t.Optional[t.Union[TrackData, PlaylistData, t.List[TrackData], TrackError]]

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
