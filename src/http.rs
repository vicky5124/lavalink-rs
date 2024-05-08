use crate::error::LavalinkResult;
use crate::model::*;

use std::sync::Arc;

use ::http::{uri::InvalidUri, Method, Uri};
use http_body_util::BodyExt;
use hyper::{body::Buf, Request};
use std::io::Read;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "python", pyo3::pyclass)]
pub struct Http {
    pub authority: String,
    pub rest_address: String,
    pub rest_address_versionless: String,
    pub headers: ::http::header::HeaderMap,
    pub request_client: Arc<
        hyper_util::client::legacy::Client<
            crate::HttpsConnector,
            http_body_util::Full<bytes::Bytes>,
        >,
    >,
}

impl Http {
    /// Makes an HTTP/1.1 request using Hyper to endpoints that return deserializable data.
    pub async fn request<R: serde::de::DeserializeOwned, T: serde::Serialize + ?Sized, U>(
        &self,
        method: Method,
        uri: U,
        data: Option<&T>,
    ) -> LavalinkResult<R>
    where
        Uri: TryFrom<U>,
        <Uri as TryFrom<U>>::Error: Into<::http::Error>,
    {
        let mut request_builder = Request::builder().method(method).uri(uri);

        {
            *request_builder.headers_mut().unwrap() = self.headers.clone()
        }

        request_builder = request_builder.header(::http::header::HOST, &self.authority);

        let request = if let Some(data) = data {
            request_builder =
                request_builder.header(::http::header::CONTENT_TYPE, "application/json");
            request_builder.body(http_body_util::Full::from(serde_json::to_vec(data)?))?
        } else {
            request_builder.body(http_body_util::Full::default())?
        };

        let response = self.request_client.request(request).await?;

        let raw_body = response.collect().await?.aggregate();
        let body = serde_json::from_reader(raw_body.reader())?;

        Ok(body)
    }

    /// Makes an HTTP/1.1 request using Hyper to endpoints that return text data.
    pub async fn raw_request<T: serde::Serialize + ?Sized, U>(
        &self,
        method: Method,
        uri: U,
        data: Option<&T>,
    ) -> LavalinkResult<String>
    where
        Uri: TryFrom<U>,
        <Uri as TryFrom<U>>::Error: Into<::http::Error>,
    {
        let mut request_builder = Request::builder().method(method).uri(uri);

        {
            *request_builder.headers_mut().unwrap() = self.headers.clone()
        }

        request_builder = request_builder.header(::http::header::HOST, &self.authority);

        let request = if let Some(data) = data {
            request_builder =
                request_builder.header(::http::header::CONTENT_TYPE, "application/json");
            request_builder.body(http_body_util::Full::from(serde_json::to_vec(data)?))?
        } else {
            request_builder.body(http_body_util::Full::default())?
        };

        let response = self.request_client.request(request).await?;

        let mut body = "".to_string();
        let raw_body = response.collect().await?.aggregate();
        raw_body.reader().read_to_string(&mut body)?;

        Ok(body)
    }

    /// Convert a path and query to a uri that points to the lavalink server.
    pub fn path_to_uri(&self, path: &str, with_version: bool) -> Result<Uri, InvalidUri> {
        if with_version {
            format!("{}{}", self.rest_address, path).try_into()
        } else {
            format!("{}{}", self.rest_address_versionless, path).try_into()
        }
    }

    /// Destroys the player for this guild in this session.
    pub async fn delete_player(
        &self,
        guild_id: impl Into<GuildId>,
        session_id: &str,
    ) -> LavalinkResult<()> {
        let guild_id = guild_id.into();

        self.raw_request(
            Method::DELETE,
            self.path_to_uri(
                &format!("/sessions/{}/players/{}", session_id, guild_id.0),
                true,
            )?,
            None::<&()>,
        )
        .await?;

        Ok(())
    }

    /// Updates or creates the player for this guild.
    pub async fn update_player(
        &self,
        guild_id: impl Into<GuildId>,
        session_id: &str,
        data: &http::UpdatePlayer,
        no_replace: bool,
    ) -> LavalinkResult<player::Player> {
        let guild_id = guild_id.into();

        let uri = self.path_to_uri(
            &format!(
                "/sessions/{}/players/{}?noReplace={}",
                session_id, guild_id.0, no_replace
            ),
            true,
        )?;

        let response = self
            .request::<crate::error::RequestResult<_>, _, _>(Method::PATCH, uri, Some(data))
            .await?
            .to_result()?;

        Ok(response)
    }

    /// Updates the session with the resuming state and timeout.
    pub async fn set_resuming_state(
        &self,
        session_id: &str,
        resuming_state: &http::ResumingState,
    ) -> LavalinkResult<http::ResumingState> {
        let response = self
            .request::<crate::error::RequestResult<_>, _, _>(
                Method::PATCH,
                self.path_to_uri(&format!("/sessions/{}", session_id), true)?,
                Some(resuming_state),
            )
            .await?
            .to_result()?;

        Ok(response)
    }

    /// Resolves audio tracks for use with the `update_player` endpoint.
    ///
    /// # Parameters
    ///
    /// - `identifier`: A track identifier.
    ///  - Can be a url: "https://youtu.be/watch?v=DrM2lo6B04I"
    ///  - A unique identifier: "DrM2lo6B04I"
    ///  - A search: "ytsearch:Ne Obliviscaris - Forget Not"
    pub async fn load_tracks(&self, identifier: &str) -> LavalinkResult<track::Track> {
        let uri = self.path_to_uri(
            &format!("/loadtracks?identifier={}", urlencoding::encode(identifier)),
            true,
        )?;

        let response = self
            .request::<crate::error::RequestResult<track::Track>, _, _>(
                Method::GET,
                uri,
                None::<&()>,
            )
            .await?
            .to_result()?;

        match response.data {
            Some(track::TrackLoadData::Error(why)) => Err(why.into()),
            _ => Ok(response),
        }
    }

    /// Request Lavalink server version.
    pub async fn version(&self) -> LavalinkResult<String> {
        let response = self
            .raw_request(
                Method::GET,
                self.path_to_uri("/version", false)?,
                None::<&()>,
            )
            .await?;

        Ok(response)
    }

    /// Request Lavalink statistics.
    ///
    /// NOTE: The frame stats will never be returned.
    pub async fn stats(&self) -> LavalinkResult<events::Stats> {
        let response = self
            .request::<crate::error::RequestResult<_>, _, _>(
                Method::GET,
                self.path_to_uri("/stats", true)?,
                None::<&()>,
            )
            .await?
            .to_result()?;

        Ok(response)
    }

    /// Request Lavalink server information.
    pub async fn info(&self) -> LavalinkResult<http::Info> {
        let response = self
            .request::<crate::error::RequestResult<_>, _, _>(
                Method::GET,
                self.path_to_uri("/info", true)?,
                None::<&()>,
            )
            .await?
            .to_result()?;

        Ok(response)
    }

    /// Decode a single track into its info.
    ///
    /// # Parameters
    ///
    /// - `track`: base64 encoded track data.
    pub async fn decode_track(&self, track: &str) -> LavalinkResult<track::TrackData> {
        let uri = self.path_to_uri(
            &format!("/decodetrack?encodedTrack={}", urlencoding::encode(track)),
            true,
        )?;

        let response = self
            .request::<crate::error::RequestResult<_>, _, _>(Method::GET, uri, None::<&()>)
            .await?
            .to_result()?;

        Ok(response)
    }

    /// Decode multiple tracks into their info.
    ///
    /// # Parameters
    ///
    /// - `tracks`: base64 encoded tracks.
    pub async fn decode_tracks(&self, tracks: &[String]) -> LavalinkResult<Vec<track::TrackData>> {
        let response = self
            .request::<crate::error::RequestResult<_>, _, _>(
                Method::POST,
                self.path_to_uri("/decodetracks", true)?,
                Some(tracks),
            )
            .await?
            .to_result()?;

        Ok(response)
    }

    /// Returns the player for this guild in this session.
    pub async fn get_player(
        &self,
        guild_id: impl Into<GuildId>,
        session_id: &str,
    ) -> LavalinkResult<player::Player> {
        let guild_id = guild_id.into();

        let response = self
            .request::<crate::error::RequestResult<_>, _, _>(
                Method::GET,
                self.path_to_uri(
                    &format!("/sessions/{}/players/{}", session_id, guild_id.0),
                    true,
                )?,
                None::<&()>,
            )
            .await?
            .to_result()?;

        Ok(response)
    }

    /// Returns a list of players in this specific session.
    pub async fn get_players(&self, session_id: &str) -> LavalinkResult<Vec<player::Player>> {
        let response = self
            .request::<crate::error::RequestResult<_>, _, _>(
                Method::GET,
                self.path_to_uri(&format!("/sessions/{}/players", session_id), true)?,
                None::<&()>,
            )
            .await?
            .to_result()?;

        Ok(response)
    }
}
