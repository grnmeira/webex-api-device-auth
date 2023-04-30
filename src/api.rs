use chrono::serde::ts_milliseconds_option;
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};

type WebsocketStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct WebexError {
    code: Option<String>,
    reason: Option<String>,
}

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

type Result<T> = std::result::Result<T, Error>;

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

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub url: Option<String>,
    pub device_type: Option<String>,
    pub name: Option<String>,
    model: Option<String>,
    localized_model: Option<String>,
    system_name: Option<String>,
    system_version: Option<String>,
    #[serde(skip_serializing, rename = "webSocketUrl")]
    pub websocket_url: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Devices {
    pub devices: Vec<Device>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub id: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "eventType")]
pub enum Data {
    #[serde(rename = "apheleia.subscription_update")]
    SubscriptionUpdate {
        subject: Option<String>,
        category: Option<String>,
        status: Option<String>,
    },
    #[serde(rename = "conversation.activity")]
    ConversationActivity { activity: Activity },
}

#[allow(dead_code)]
#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: Option<String>,
    pub data: Option<Data>,
    pub filter_message: Option<bool>,
    //pub headers: Option<String>,
    pub sequence_number: Option<u32>,
    #[serde(with = "ts_milliseconds_option")]
    pub timestamp: Option<DateTime<Utc>>,
    pub tracking_id: Option<String>,
}

pub struct EventListener {
    stream: WebsocketStream,
}

impl EventListener {
    fn new(stream: WebsocketStream) -> EventListener {
        EventListener { stream }
    }

    pub async fn next(&mut self) -> Result<Event> {
        while let Some(Ok(message)) = self.stream.next().await {
            match message {
                Message::Ping(data) => {
                    self.stream.send(Message::Pong(data)).await?;
                }
                Message::Binary(data) => {
                    println!("{:#?}", &String::from_utf8_lossy(&data));
                    return String::from_utf8(data)
                        .map_err(|err| Error::JsonParsingError(err.to_string()))
                        .and_then(|string| {
                            serde_json::from_str::<Event>(&string)
                                .map_err(|err| Error::JsonParsingError(err.to_string()))
                        });
                }
                _ => (),
            }
        }

        Err(Error::WebsocketError(
            "failed to read event from stream".to_owned(),
        ))
    }
}

pub struct Client {
    device_id: Option<String>,
    bearer_token: String,
    reqwest_client: reqwest::Client,
}

impl Client {
    pub fn new(bearer_token: &str, device_id: Option<&str>) -> Client {
        Client {
            bearer_token: bearer_token.to_owned(),
            reqwest_client: reqwest::Client::new(),
            device_id: device_id.map(|id| id.to_owned()),
        }
    }

    pub async fn listen_to_events(&self) -> Result<EventListener> {
        let device = match &self.device_id {
            Some(device_id) => self.get_device(device_id).await?,
            _ => self.post_devices().await?,
        };

        let Some(websocket_url) = device.websocket_url.as_ref() else {
            return Err(Error::GenericError("device has no Websocket URL".to_owned()));
        };

        let url = url::Url::parse(websocket_url).or(Err(Error::GenericError(format!(
            "parsing URL {}",
            &websocket_url
        ))))?;

        let host = url.host_str().ok_or(Error::GenericError(format!(
            "unable to obtain host from URL {}",
            url
        )))?;

        let request = http::Request::builder()
            .uri(websocket_url)
            .header("Authorization", format!("Bearer {}", self.bearer_token))
            .header(
                "Sec-Websocket-Key",
                tungstenite::handshake::client::generate_key(),
            )
            .header("Sec-Websocket-Version", "13")
            .header("Connection", "Upgrade")
            .header("Host", host.to_string())
            .header("Upgrade", "websocket")
            .body(())
            .unwrap();

        let (ws_stream, _) = connect_async(request).await?;

        Ok(EventListener::new(ws_stream))
    }

    pub async fn get_device(&self, device_id: &str) -> Result<Device> {
        let response = self
            .reqwest_client
            .get(format!(
                "https://wdm-a.wbx2.com/wdm/api/v1/devices/{}",
                device_id
            ))
            .header("Accept", "application/json")
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        if response.status().is_success() {
            let json_result = response.json::<Device>().await?;
            return Ok(json_result);
        }

        match response.status().as_u16() {
            code @ 400..=499 => Err(Error::HttpStatus(code, None)),
            error_code => Err(Error::HttpStatus(error_code, None)),
        }
    }

    pub async fn get_devices(&self) -> Result<Devices> {
        let response = self
            .reqwest_client
            .get("https://wdm-a.wbx2.com/wdm/api/v1/devices")
            .header("Accept", "application/json")
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        if response.status().is_success() {
            let json_result = response.json::<Devices>().await?;
            return Ok(json_result);
        }

        match response.status().as_u16() {
            code @ 400..=499 => Err(Error::HttpStatus(code, None)),
            error_code => Err(Error::HttpStatus(error_code, None)),
        }
    }

    pub async fn post_devices(&self) -> Result<Device> {
        let device_object = Device {
            device_type: Some("UNKNOWN".to_string()),
            name: Some("pixoo-integration".to_string()),
            model: Some("pixoo-64".to_string()),
            localized_model: Some("".to_string()),
            system_name: Some("Windows".to_string()),
            system_version: Some("10".to_string()),
            ..Default::default()
        };

        let response = self
            .reqwest_client
            .post("https://wdm-a.wbx2.com/wdm/api/v1/devices")
            .json(&device_object)
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        if response.status().is_success() {
            let json_result = response.json::<Device>().await?;
            return Ok(json_result);
        }

        match response.status().as_u16() {
            code @ 400..=499 => Err(Error::HttpStatus(code, None)),
            error_code => Err(Error::HttpStatus(error_code, None)),
        }
    }

    pub async fn delete_device(&self, device_url: &str) -> Result<()> {
        self.reqwest_client
            .delete(device_url)
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        Ok(())
    }
}
