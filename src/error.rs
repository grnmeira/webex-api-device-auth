use crate::dto::WebexError;
use tokio_tungstenite::tungstenite;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("HTTP error with status: {0}")]
    HttpStatus(u16, Option<WebexError>),
    #[error("JSON parsing error: {0}")]
    JsonParsingError(String),
    #[error("generic error: {0}")]
    GenericError(String),
    #[error("websocket error: {0}")]
    WebsocketError(String),
}

pub(crate) type Result<T> = std::result::Result<T, Error>;

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        println!("{:#?}", e);
        if let Some(status_code) = e.status() {
            Error::HttpStatus(status_code.as_u16(), None)
        } else if e.is_decode() {
            Error::JsonParsingError(format!("{}", e))
        } else {
            Error::GenericError(format!("{}", e))
        }
    }
}

impl From<tungstenite::error::Error> for Error {
    fn from(e: tungstenite::error::Error) -> Self {
        match &e {
            tungstenite::error::Error::Http(response) => {
                if let Some(body) = response.body() {
                    Error::GenericError(String::from_utf8_lossy(body).to_string())
                } else {
                    Error::GenericError(format!("HTTP error from websocket: {}", e))
                }
            }
            _ => Error::WebsocketError(e.to_string()),
        }
    }
}
