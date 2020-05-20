use std::{
    error::Error,
    fmt::{
        Display,
        Formatter,
        Result,
    },
};
use async_tungstenite::tungstenite::error::Error as TungsteniteError;

#[derive(Debug)]
pub enum LavalinkError {
    NoWebsocket,
    MissingHandlerToken,
    MissingHandlerEndpoint,
    MissingHandlerSessionId,
    InvalidDataToVoiceUpdate,
    InvalidDataToPlay,
    InvalidDataToStop,
    InvalidDataToDestroy,
    InvalidDataToPause,
    InvalidDataToVolume,
    InvalidDataToSeek,
    ErrorSendingVoiceUpdatePayload(TungsteniteError),
    ErrorSendingPlayPayload(TungsteniteError),
    ErrorSendingStopPayload(TungsteniteError),
    ErrorSendingDestroyPayload(TungsteniteError),
    ErrorSendingPausePayload(TungsteniteError),
    ErrorSendingVolumePayload(TungsteniteError),
    ErrorSendingSeekPayload(TungsteniteError),
}

impl Error for LavalinkError {}

impl Display for LavalinkError {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            LavalinkError::NoWebsocket => write!(f, "There is no initialized websocket."),
            LavalinkError::MissingHandlerToken => write!(f, "No `token` was found on the handler."),
            LavalinkError::MissingHandlerEndpoint => write!(f, "No `endpoint` was found on the handler."),
            LavalinkError::MissingHandlerSessionId => write!(f, "No `session_id` was found on the hander"),
            LavalinkError::InvalidDataToPlay => write!(f, "Invalid data was provided to the `play` json."),
            LavalinkError::InvalidDataToStop => write!(f, "Invalid data was provided to the `stop` json."),
            LavalinkError::InvalidDataToDestroy => write!(f, "Invalid data was provided to the `destroy` json."),
            LavalinkError::InvalidDataToPause => write!(f, "Invalid data was provided to the `pause` json."),
            LavalinkError::InvalidDataToVolume => write!(f, "Invalid data was provided to the `volume` json."),
            LavalinkError::InvalidDataToSeek => write!(f, "Invalid data was provided to the `seek` json."),
            LavalinkError::InvalidDataToVoiceUpdate => write!(f, "Invalid data was provided to the `voiceUpdate` json."),
            LavalinkError::ErrorSendingPlayPayload(why) => write!(f, "Error while sending payload `play` json => {:?}", why),
            LavalinkError::ErrorSendingStopPayload(why) => write!(f, "Error while sending payload `stop` json => {:?}", why),
            LavalinkError::ErrorSendingDestroyPayload(why) => write!(f, "Error while sending payload `destroy` json => {:?}", why),
            LavalinkError::ErrorSendingPausePayload(why) => write!(f, "Error while sending payload `pause` json => {:?}", why),
            LavalinkError::ErrorSendingVolumePayload(why) => write!(f, "Error while sending payload `volume` json => {:?}", why),
            LavalinkError::ErrorSendingSeekPayload(why) => write!(f, "Error while sending payload `seek` json => {:?}", why),
            LavalinkError::ErrorSendingVoiceUpdatePayload(why) => write!(f, "Error while sending payload `voiceUpdate` json => {:?}", why),
            //_ => write!(f, "Unhandled error occurred."),
        }
    }
}
