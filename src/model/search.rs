#![allow(clippy::large_enum_variant)]

use crate::error::LavalinkResult;

/// Search engines supported by Lavalink and LavaSrc.
pub enum SearchEngines {
    YouTube,
    YouTubeMusic,
    SoundCloud,
    /// NOTE: Requires LavaSrc plugin.
    Spotify,
    /// NOTE: Requires LavaSrc plugin.
    SpotifyRecommended(SpotifyRecommendedParameters),
    /// NOTE: Requires LavaSrc plugin.
    AppleMusic,
    /// NOTE: Requires LavaSrc plugin.
    Deezer,
    /// NOTE: Requires LavaSrc plugin.
    DeezerISRC,
    /// NOTE: Requires LavaSrc plugin.
    YandexMusic,
    /// NOTE: Requires LavaSrc plugin.
    FloweryTTS(FloweryTTSParameters),
}

impl ToString for SearchEngines {
    fn to_string(&self) -> String {
        use SearchEngines::*;
        match self {
            YouTube => "ytsearch".to_string(),
            YouTubeMusic => "ytmsearch".to_string(),
            SoundCloud => "scsearch".to_string(),
            Spotify => "spsearch".to_string(),
            SpotifyRecommended(_) => "sprec".to_string(),
            AppleMusic => "amsearch".to_string(),
            Deezer => "dzsearch".to_string(),
            DeezerISRC => "dzisrc".to_string(),
            YandexMusic => "ymsearch".to_string(),
            FloweryTTS(_) => "ftts://".to_string(),
        }
    }
}

impl SearchEngines {
    /// Create a String you can pip to `load_tracks()` to get the search results.
    ///
    /// Example:
    /// ```rust,untested
    /// let query = SearchEngines::YouTubeMusic.to_query("Ne Obliviscaris - Forget Not").unwrap();
    /// lavalink_client.load_tracks(guild_id, query).await?;
    /// ```
    pub fn to_query(&self, base_query: &str) -> LavalinkResult<String> {
        use SearchEngines::*;
        match self {
            YouTube | YouTubeMusic | SoundCloud | Spotify | AppleMusic | Deezer | DeezerISRC
            | YandexMusic => Ok(format!("{}:{}", self.to_string(), base_query)),
            SpotifyRecommended(x) => {
                let query = serde_qs::to_string(&x)?;
                Ok(format!("{}{}?{}", self.to_string(), base_query, query))
            }
            FloweryTTS(x) => {
                let query = serde_qs::to_string(&x)?;
                Ok(format!("{}{}?{}", self.to_string(), base_query, query))
            }
        }
    }
}

/// Any of the seed fields must have a value.
///
/// Spotify documentation can be found [here](https://developer.spotify.com/documentation/web-api/reference/get-recommendations)
#[derive(Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all, set_all))]
pub struct SpotifyRecommendedParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed_artists: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed_genres: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed_tracks: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_acousticness: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_acousticness: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_acousticness: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_danceability: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_danceability: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_danceability: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_energy: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_energy: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_energy: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_instrumentalness: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_instrumentalness: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_instrumentalness: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_key: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_key: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_key: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_liveness: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_liveness: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_liveness: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_loudness: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_loudness: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_loudness: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_mode: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_mode: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_mode: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_popularity: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_popularity: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_popularity: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_speechiness: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_speechiness: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_speechiness: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_tempo: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tempo: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_tempo: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_time_signature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_time_signature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_time_signature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_valence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_valence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_valence: Option<f64>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[cfg_attr(feature = "python", pyo3::pyclass(get_all, set_all))]
pub struct FloweryTTSParameters {
    /// A list of voices can be found [here](https://api.flowery.pw/v1/tts/voices)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<String>,
    /// TODO: Document this.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translate: Option<bool>,
    /// The silence parameter is in milliseconds. Range is 0 to 10000. The default is 0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub silence: Option<u16>,
    /// Supported formats are: mp3, ogg_opus, ogg_vorbis, aac, wav, and flac.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_format: Option<String>,
    /// The speed parameter is a float between 0.5 and 10. The default is 1.0. (0.5 is half speed, 2.0 is double speed, etc.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
}
