use std::{
    error::Error,
    fmt::{Display, Formatter, Result},
};

#[cfg(feature = "tokio-02-marker")]
use async_tungstenite_compat as async_tungstenite;
#[cfg(feature = "tokio-02-marker")]
use reqwest_compat as reqwest;

use async_tungstenite::tungstenite::error::Error as TungsteniteError;
use reqwest::{header::InvalidHeaderValue, Error as ReqwestError};

#[derive(Debug)]
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
                write!(f, "Please, call client.create_session() for this method to work correctly.")
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
