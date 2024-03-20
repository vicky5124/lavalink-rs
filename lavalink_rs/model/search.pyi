class SearchEngines:
    @staticmethod
    def youtube(query: str) -> str: ...
    @staticmethod
    def youtube_music(query: str) -> str: ...
    @staticmethod
    def soundcloud(query: str) -> str: ...
    @staticmethod
    def spotify(query: str) -> str: ...
    @staticmethod
    def spotify_recommended(
        query: str,
        parameters: SpotifyRecommendedParameters,
    ) -> str: ...
    @staticmethod
    def apple_music(query: str) -> str: ...
    @staticmethod
    def deezer(query: str) -> str: ...
    @staticmethod
    def deezer_isrc(query: str) -> str: ...
    @staticmethod
    def yandex_music(query: str) -> str: ...
    @staticmethod
    def flowery_tts(
        query: str,
        parameters: FloweryTTSParameters,
    ) -> str: ...

class SpotifyRecommendedParameters:
    seed_artists: str
    seed_genres: str
    seed_tracks: str
    limit: int
    market: str
    min_acousticness: float
    max_acousticness: float
    target_acousticness: float
    min_danceability: float
    max_danceability: float
    target_danceability: float
    min_duration_ms: int
    max_duration_ms: int
    target_duration_ms: int
    min_energy: float
    max_energy: float
    target_energy: float
    min_instrumentalness: float
    max_instrumentalness: float
    target_instrumentalness: float
    min_key: float
    max_key: float
    target_key: float
    min_liveness: float
    max_liveness: float
    target_liveness: float
    min_loudness: int
    max_loudness: int
    target_loudness: int
    min_mode: float
    max_mode: float
    target_mode: float
    min_popularity: int
    max_popularity: int
    target_popularity: int
    min_speechiness: float
    max_speechiness: float
    target_speechiness: float
    min_tempo: int
    max_tempo: int
    target_tempo: int
    min_time_signature: float
    max_time_signature: float
    target_time_signature: float
    min_valence: float
    max_valence: float
    target_valence: float

class FloweryTTSParameters:
    voice: str
    translate: bool
    silence: int
    audio_format: str
    speed: float
