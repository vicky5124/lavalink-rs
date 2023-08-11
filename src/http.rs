use crate::error::LavalinkResult;
use crate::model::*;

use reqwest::Client as ReqwestClient;
use url::Url;

#[derive(Debug, Clone)]
pub struct Http {
    pub rest_address: String,
    pub rest_address_versionless: String,
    pub rest_client: ReqwestClient,
}

impl Http {
    pub async fn delete_player(&self, guild_id: GuildId, session_id: &str) -> LavalinkResult<()> {
        self.rest_client
            .delete(format!(
                "{}/sessions/{}/players/{}",
                self.rest_address, session_id, guild_id.0
            ))
            .send()
            .await?;

        Ok(())
    }

    pub async fn update_player(
        &self,
        guild_id: GuildId,
        session_id: &str,
        data: &http::UpdatePlayer,
        no_replace: bool,
    ) -> LavalinkResult<player::Player> {
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

    pub async fn load_tracks(&self, term: &str) -> LavalinkResult<track::Track> {
        let url = Url::parse_with_params(
            &format!("{}/loadtracks", self.rest_address),
            &[("identifier", &term)],
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

    pub async fn get_player(
        &self,
        guild_id: GuildId,
        session_id: &str,
    ) -> LavalinkResult<player::Player> {
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
