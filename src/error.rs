use crate::model::track::TrackError;

use std::error::Error;
use std::fmt::{Display, Formatter, Result};
use std::io::Error as IoError;

use ::http::header::InvalidHeaderValue;
use ::http::method::InvalidMethod;
use ::http::uri::InvalidUri;
use ::http::Error as HttpError;
use hyper::Error as HyperError;
use hyper_util::client::legacy::Error as HyperClientError;
use oneshot::RecvError;
use tokio::sync::mpsc::error::SendError;
#[cfg(feature = "_tungstenite")]
use tokio_tungstenite::tungstenite::error::Error as TungsteniteError;
#[cfg(feature = "_websockets")]
use tokio_websockets::error::Error as WebsocketsError;

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
#[non_exhaustive]
/// Every error the library can return.
pub enum LavalinkError {
    IoError(IoError),
    #[cfg(feature = "_tungstenite")]
    WebsocketError(TungsteniteError),
    #[cfg(feature = "_websockets")]
    WebsocketError(WebsocketsError),
    InvalidHeaderValue(InvalidHeaderValue),
    HyperError(HyperError),
    HyperClientError(HyperClientError),
    HttpError(HttpError),
    InvalidUri(InvalidUri),
    InvalidMethod(InvalidMethod),
    ChannelSendError,
    ChannelReceiveError(RecvError),
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
            LavalinkError::IoError(why) => {
                write!(f, "I/O Error => {:?}", why)
            }
            LavalinkError::InvalidHeaderValue(why) => {
                write!(f, "Invalid Header Value => {:?}", why)
            }
            LavalinkError::HttpError(why) => {
                write!(f, "HttpError => {:?}", why)
            }
            LavalinkError::InvalidUri(why) => {
                write!(f, "Invalid URI => {:?}", why)
            }
            LavalinkError::InvalidMethod(why) => {
                write!(f, "Invalid HTTP request method => {:?}", why)
            }
            LavalinkError::HyperError(why) => {
                write!(f, "Hyper Error => {:?}", why)
            }
            LavalinkError::HyperClientError(why) => {
                write!(f, "Hyper Client Error => {:?}", why)
            }
            LavalinkError::ChannelSendError => {
                write!(f, "The channel receiver is closed.")
            }
            LavalinkError::ChannelReceiveError(why) => {
                write!(f, "Error receiving from player context: {:?}", why)
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

impl From<IoError> for LavalinkError {
    fn from(err: IoError) -> LavalinkError {
        LavalinkError::IoError(err)
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

#[cfg(feature = "_tungstenite")]
impl From<TungsteniteError> for LavalinkError {
    fn from(err: TungsteniteError) -> LavalinkError {
        LavalinkError::WebsocketError(err)
    }
}

#[cfg(feature = "_websockets")]
impl From<WebsocketsError> for LavalinkError {
    fn from(err: WebsocketsError) -> LavalinkError {
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

impl From<InvalidUri> for LavalinkError {
    fn from(err: InvalidUri) -> LavalinkError {
        LavalinkError::InvalidUri(err)
    }
}

impl From<InvalidMethod> for LavalinkError {
    fn from(err: InvalidMethod) -> LavalinkError {
        LavalinkError::InvalidMethod(err)
    }
}

impl From<HyperError> for LavalinkError {
    fn from(err: HyperError) -> LavalinkError {
        LavalinkError::HyperError(err)
    }
}

impl From<HyperClientError> for LavalinkError {
    fn from(err: HyperClientError) -> LavalinkError {
        LavalinkError::HyperClientError(err)
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
