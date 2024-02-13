use crate::model::track::TrackError;

use std::error::Error;
use std::fmt::{Display, Formatter, Result};

use http::header::InvalidHeaderValue;
use http::Error as HttpError;
use oneshot::RecvError;
use reqwest::Error as ReqwestError;
use tokio::sync::mpsc::error::SendError;
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;
use url::ParseError;

#[cfg(feature = "python")]
use pyo3::exceptions::PyException;
#[cfg(feature = "python")]
use pyo3::PyErr;

pub type LavalinkResult<T> = std::result::Result<T, LavalinkError>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Response sent by REST requests when there's an error.
pub struct ResponseError {
    pub status: u16,
    pub timestamp: u64,
    pub error: String,
    pub message: String,
    pub path: String,
    pub trace: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub(crate) enum RequestResult<T> {
    Ok(T),
    Err(ResponseError),
}

impl<T> RequestResult<T> {
    pub fn to_result(self) -> std::result::Result<T, ResponseError> {
        match self {
            Self::Ok(x) => Ok(x),
            Self::Err(x) => Err(x),
        }
    }
}

#[derive(Debug)]
/// Every error the library can return.
pub enum LavalinkError {
    WebsocketError(TungsteniteError),
    InvalidHeaderValue(InvalidHeaderValue),
    ReqwestError(ReqwestError),
    HttpError(HttpError),
    ChannelSendError,
    ChannelReceiveError(RecvError),
    UrlParseError(ParseError),
    SerdeErrorQs(serde_qs::Error),
    SerdeErrorJson(serde_json::Error),

    ResponseError(ResponseError),
    NoSessionPresent,
    TrackError(TrackError),
    InvalidDataType,
    Timeout,
}

impl Error for LavalinkError {}

impl Display for LavalinkError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            LavalinkError::WebsocketError(why) => {
                write!(
                    f,
                    "Error while sending payload to the websocket => {:?}",
                    why
                )
            }
            LavalinkError::InvalidHeaderValue(why) => {
                write!(f, "Invalid Header Value => {:?}", why)
            }
            LavalinkError::HttpError(why) => {
                write!(f, "HttpError => {:?}", why)
            }
            LavalinkError::ReqwestError(why) => {
                write!(f, "Reqwest Error => {:?}", why)
            }
            LavalinkError::ChannelSendError => {
                write!(f, "The channel receiver is closed.")
            }
            LavalinkError::ChannelReceiveError(why) => {
                write!(f, "Error receiving from player context: {:?}", why)
            }
            LavalinkError::UrlParseError(why) => {
                write!(f, "Url Parsing Error => {:?}", why)
            }
            LavalinkError::SerdeErrorQs(why) => {
                write!(f, "Error serializing or desesrializing qs => {:?}", why)
            }
            LavalinkError::SerdeErrorJson(why) => {
                write!(f, "Error serializing or desesrializing json => {:?}", why)
            }

            LavalinkError::NoSessionPresent => {
                write!(
                    f,
                    "Please, call client.create_session() for this method to work correctly."
                )
            }
            LavalinkError::ResponseError(why) => {
                write!(f, "Error from lavalink server: {:?}", why)
            }
            LavalinkError::TrackError(why) => {
                write!(f, "Error loading tracks: {:?}", why)
            }
            LavalinkError::InvalidDataType => {
                write!(f, "The value type provided does not match the data type id, or no data was ever provided.")
            }
            LavalinkError::Timeout => {
                write!(f, "Timeout reached while waiting for response.")
            }
        }
    }
}

impl From<ResponseError> for LavalinkError {
    fn from(err: ResponseError) -> LavalinkError {
        LavalinkError::ResponseError(err)
    }
}

impl From<TrackError> for LavalinkError {
    fn from(err: TrackError) -> LavalinkError {
        LavalinkError::TrackError(err)
    }
}

impl From<TungsteniteError> for LavalinkError {
    fn from(err: TungsteniteError) -> LavalinkError {
        LavalinkError::WebsocketError(err)
    }
}

impl From<InvalidHeaderValue> for LavalinkError {
    fn from(err: InvalidHeaderValue) -> LavalinkError {
        LavalinkError::InvalidHeaderValue(err)
    }
}

impl From<HttpError> for LavalinkError {
    fn from(err: HttpError) -> LavalinkError {
        LavalinkError::HttpError(err)
    }
}

impl From<ReqwestError> for LavalinkError {
    fn from(err: ReqwestError) -> LavalinkError {
        LavalinkError::ReqwestError(err)
    }
}

impl From<ParseError> for LavalinkError {
    fn from(err: ParseError) -> LavalinkError {
        LavalinkError::UrlParseError(err)
    }
}

impl<T> From<SendError<T>> for LavalinkError {
    fn from(_: SendError<T>) -> LavalinkError {
        LavalinkError::ChannelSendError
    }
}

impl From<RecvError> for LavalinkError {
    fn from(err: RecvError) -> LavalinkError {
        LavalinkError::ChannelReceiveError(err)
    }
}

impl From<serde_qs::Error> for LavalinkError {
    fn from(err: serde_qs::Error) -> Self {
        LavalinkError::SerdeErrorQs(err)
    }
}

impl From<serde_json::Error> for LavalinkError {
    fn from(err: serde_json::Error) -> Self {
        LavalinkError::SerdeErrorJson(err)
    }
}

#[cfg(feature = "python")]
impl From<LavalinkError> for PyErr {
    fn from(err: LavalinkError) -> PyErr {
        error!("{}", err);
        PyErr::new::<PyException, _>(format!("{:?}", err))
    }
}
