use crate::error::LavalinkResult;
use crate::model::*;

use reqwest::Client as ReqwestClient;
use reqwest::Method;
use url::Url;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "python", pyo3::pyclass)]
pub struct Http {
    pub rest_address: String,
    pub rest_address_versionless: String,
    pub rest_client: ReqwestClient,
}

impl Http {
    /// Makes a raw request to the Lavalink REST API
    ///
    /// # Example:
    ///
    /// ```rust
    /// use ::http::Method;
    ///
    /// let node = lavalink_client.get_node_for_guild(guild_id);
    /// let path = format!("/v4/sessions/{}/players/{}", &node.session_id.load(), guild_id.0);
    /// let result = node.http.raw_request(Method::PATCH, path, &serde_json::json!({"encodedTrack": null})).await?;
    /// let player = serde_json::from_value::<lavalink_rs::error::RequestResult<lavalink_rs::player::Player>>(result)?.to_result()?;
    /// ```
    pub async fn raw_request(
        &self,
        method: Method,
        path: String,
        data: &serde_json::Value,
    ) -> LavalinkResult<serde_json::Value> {
        let url = format!("{}{}", self.rest_address_versionless, path);

        let response = self
            .rest_client
            .request(method, url)
            .json(data)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        Ok(response)
    }

    /// Destroys the player for this guild in this session.
    pub async fn delete_player(
        &self,
        guild_id: impl Into<GuildId>,
        session_id: &str,
    ) -> LavalinkResult<()> {
        let guild_id = guild_id.into();

        self.rest_client
            .delete(format!(
                "{}/sessions/{}/players/{}",
                self.rest_address, session_id, guild_id.0
            ))
            .send()
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

        let url = Url::parse_with_params(
            &format!(
                "{}/sessions/{}/players/{}",
                self.rest_address, session_id, guild_id.0
            ),
            &[("noReplace", &no_replace.to_string())],
        )?;

        let response = self
            .rest_client
            .patch(url)
            .json(data)
            .send()
            .await?
            .json::<crate::error::RequestResult<player::Player>>()
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
            .rest_client
            .patch(format!("{}/sessions/{}", self.rest_address, session_id))
            .json(resuming_state)
            .send()
            .await?
            .json::<crate::error::RequestResult<http::ResumingState>>()
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
        let url = Url::parse_with_params(
            &format!("{}/loadtracks", self.rest_address),
            &[("identifier", &identifier)],
        )?;

        let result = self
            .rest_client
            .get(url)
            .send()
            .await?
            .json::<crate::error::RequestResult<track::Track>>()
            .await?
            .to_result()?;

        match result.data {
            Some(track::TrackLoadData::Error(why)) => Err(why.into()),
            _ => Ok(result),
        }
    }

    /// Request Lavalink server version.
    pub async fn version(&self) -> LavalinkResult<String> {
        let response = self
            .rest_client
            .get(format!("{}/version", self.rest_address_versionless))
            .send()
            .await?
            .text()
            .await?;

        Ok(response)
    }

    /// Request Lavalink statistics.
    ///
    /// NOTE: The frame stats will never be returned.
    pub async fn stats(&self) -> LavalinkResult<events::Stats> {
        let response = self
            .rest_client
            .get(format!("{}/stats", self.rest_address))
            .send()
            .await?
            .json::<crate::error::RequestResult<events::Stats>>()
            .await?
            .to_result()?;

        Ok(response)
    }

    /// Request Lavalink server information.
    pub async fn info(&self) -> LavalinkResult<http::Info> {
        let response = self
            .rest_client
            .get(format!("{}/info", self.rest_address))
            .send()
            .await?
            .json::<crate::error::RequestResult<http::Info>>()
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
        let url = Url::parse_with_params(
            &format!("{}/decodetrack", self.rest_address),
            &[("encodedTrack", &track)],
        )?;

        let response = self
            .rest_client
            .get(url)
            .send()
            .await?
            .json::<crate::error::RequestResult<track::TrackData>>()
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
            .rest_client
            .post(format!("{}/decodetracks", self.rest_address))
            .json(tracks)
            .send()
            .await?
            .json::<crate::error::RequestResult<Vec<track::TrackData>>>()
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
            .rest_client
            .get(format!(
                "{}/sessions/{}/players/{}",
                self.rest_address, session_id, guild_id.0
            ))
            .send()
            .await?
            .json::<crate::error::RequestResult<player::Player>>()
            .await?
            .to_result()?;

        Ok(response)
    }

    /// Returns a list of players in this specific session.
    pub async fn get_players(&self, session_id: &str) -> LavalinkResult<Vec<player::Player>> {
        let response = self
            .rest_client
            .get(format!(
                "{}/sessions/{}/players",
                self.rest_address, session_id
            ))
            .send()
            .await?
            .json::<crate::error::RequestResult<Vec<player::Player>>>()
            .await?
            .to_result()?;

        Ok(response)
    }
}
