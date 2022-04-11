use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
};

use async_tungstenite::tungstenite::error::Error as TungsteniteError;
use reqwest::{header::InvalidHeaderValue, Error as ReqwestError};
use tokio::sync::mpsc::error::SendError;

pub type LavalinkResult<T> = std::result::Result<T, LavalinkError>;

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum LavalinkError {
    /// TungsteniteError redirect.
    ErrorWebsocketPayload(TungsteniteError),
    /// Invalid Headers redirect.
    InvalidHeaderValue(InvalidHeaderValue),
    /// ReqwestError redirect.
    ReqwestError(ReqwestError),
    /// Returned by [`PlayParameters::queue`] if no queue is present.
    ///
    /// [`PlayParameters::queue`]: crate::builders::PlayParameters
    NoSessionPresent,
    /// When joining a voice channel times out.
    #[cfg(feature = "discord-gateway")]
    Timeout,
    #[cfg(feature = "discord-gateway")]
    MissingConnectionField(&'static str),
    MissingLavalinkSocket,
    ChannelSendError,
}

impl Error for LavalinkError {}

impl Display for LavalinkError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            LavalinkError::ErrorWebsocketPayload(why) => {
                write!(
                    f,
                    "Error while sending payload to the websocket => {:?}",
                    why
                )
            }
            LavalinkError::InvalidHeaderValue(why) => {
                write!(f, "Invalid Header Value => {:?}", why)
            }
            LavalinkError::ReqwestError(why) => {
                write!(f, "Reqwest Error => {:?}", why)
            }
            LavalinkError::NoSessionPresent => {
                write!(
                    f,
                    "Please, call client.create_session() for this method to work correctly."
                )
            }
            #[cfg(feature = "discord-gateway")]
            LavalinkError::Timeout => {
                write!(f, "Joining the voice channel timed out.")
            }
            #[cfg(feature = "discord-gateway")]
            &LavalinkError::MissingConnectionField(field) => {
                write!(f, "Gateway connection is missing the field `{}`", field)
            }
            LavalinkError::MissingLavalinkSocket => {
                write!(f, "Initialize a lavalink websocket connection.")
            }
            LavalinkError::ChannelSendError => {
                write!(f, "The channel receiver is closed.")
            }
        }
    }
}

impl From<TungsteniteError> for LavalinkError {
    fn from(err: TungsteniteError) -> LavalinkError {
        LavalinkError::ErrorWebsocketPayload(err)
    }
}

impl From<InvalidHeaderValue> for LavalinkError {
    fn from(err: InvalidHeaderValue) -> LavalinkError {
        LavalinkError::InvalidHeaderValue(err)
    }
}

impl From<ReqwestError> for LavalinkError {
    fn from(err: ReqwestError) -> LavalinkError {
        LavalinkError::ReqwestError(err)
    }
}

impl<T> From<SendError<T>> for LavalinkError {
    fn from(_: SendError<T>) -> LavalinkError {
        LavalinkError::ChannelSendError
    }
}
