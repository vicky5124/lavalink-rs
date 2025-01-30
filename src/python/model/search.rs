use crate::error::LavalinkError;
use crate::model::search::*;

use pyo3::prelude::*;

#[pymodule]
pub fn search(_py: Python<'_>, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<SearchEngines>()?;
    m.add_class::<SpotifyRecommendedParameters>()?;
    m.add_class::<FloweryTTSParameters>()?;

    Ok(())
}

#[pymethods]
impl SpotifyRecommendedParameters {
    #[new]
    fn new_py() -> SpotifyRecommendedParameters {
        SpotifyRecommendedParameters::default()
    }
}

#[pymethods]
impl FloweryTTSParameters {
    #[new]
    fn new_py() -> FloweryTTSParameters {
        FloweryTTSParameters::default()
    }
}

#[pyclass]
pub(crate) struct SearchEngines;

#[pymethods]
impl SearchEngines {
    #[staticmethod]
    fn youtube(query: String) -> String {
        crate::model::search::SearchEngines::YouTube
            .to_query(&query)
            .unwrap()
    }
    #[staticmethod]
    fn youtube_music(query: String) -> String {
        crate::model::search::SearchEngines::YouTubeMusic
            .to_query(&query)
            .unwrap()
    }
    #[staticmethod]
    fn soundcloud(query: String) -> String {
        crate::model::search::SearchEngines::SoundCloud
            .to_query(&query)
            .unwrap()
    }
    #[staticmethod]
    fn spotify(query: String) -> String {
        crate::model::search::SearchEngines::Spotify
            .to_query(&query)
            .unwrap()
    }
    #[staticmethod]
    fn spotify_recommended(
        query: String,
        parameters: SpotifyRecommendedParameters,
    ) -> Result<String, LavalinkError> {
        crate::model::search::SearchEngines::SpotifyRecommended(parameters).to_query(&query)
    }
    #[staticmethod]
    fn apple_music(query: String) -> String {
        crate::model::search::SearchEngines::AppleMusic
            .to_query(&query)
            .unwrap()
    }
    #[staticmethod]
    fn deezer(query: String) -> String {
        crate::model::search::SearchEngines::Deezer
            .to_query(&query)
            .unwrap()
    }
    #[staticmethod]
    fn deezer_isrc(query: String) -> String {
        crate::model::search::SearchEngines::DeezerISRC
            .to_query(&query)
            .unwrap()
    }
    #[staticmethod]
    fn yandex_music(query: String) -> String {
        crate::model::search::SearchEngines::YandexMusic
            .to_query(&query)
            .unwrap()
    }
    #[staticmethod]
    fn flowery_tts(
        query: String,
        parameters: FloweryTTSParameters,
    ) -> Result<String, LavalinkError> {
        crate::model::search::SearchEngines::FloweryTTS(parameters).to_query(&query)
    }
}
