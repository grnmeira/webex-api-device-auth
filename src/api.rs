use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("HTTP error with status: {0}")]
    HttpStatus(u16, Option<WebexError>),
    #[error("JSON parsing error: {0}")]
    JsonParsingError(String),
    #[error("generic error: {0}")]
    GenericError(String),
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

#[derive(Deserialize, Debug)]
pub struct WebexError {
    code: Option<String>,
    reason: Option<String>,
}

pub struct EventListener {
    bearer_token: String,
    device_id: Option<String>
}

impl EventListener {
    pub fn new(bearer_token: &str) -> EventListener {
        EventListener { 
            bearer_token: bearer_token.to_owned(), 
            device_id: None
        }
    }

    pub fn new_with_existing_device(bearer_token: &str, device_id: &str) -> EventListener {
        EventListener { 
            bearer_token: bearer_token.to_owned(), 
            device_id: Some(device_id.to_owned())
        }
    }
}

pub struct Client {
    bearer_token: String,
    reqwest_client: reqwest::Client,
}

impl Client {
    pub fn new(bearer_token: &str) -> Client {
        Client {
            bearer_token: bearer_token.to_owned(),
            reqwest_client: reqwest::Client::new(),
        }
    }

    pub async fn listen_to_events(&self, device_id: Option<&str>) -> Result<EventListener> {
        if let Some(device_id) = device_id {
            let device = self.get_device(device_id).await?;

            let websocket_url = match device.websocket_url.as_ref() {
                Some(url) => url,
                None => return Err(Error::GenericError("device has no Websocket URL".to_owned()))
            };

            let host = match url::Host::parse(&websocket_url) {
                Ok(host) => host,
                Err(_) => return Err(Error::GenericError(format!("unable to extract host from URL: {}", websocket_url)))
            };

            let request = http::Request::builder()
                .uri(websocket_url)
                .header("Authorization", format!("Bearer {}", self.bearer_token))
                .header("Sec-Websocket-Key", tungstenite::handshake::client::generate_key())
                .header("Sec-Websocket-Version", "13")
                .header("Connection", "Upgrade")
                .header("Host", host.to_string())
                .header("Upgrade", "websocket")
                .body(())
                .unwrap();

            return Ok(EventListener::new(&self.bearer_token))
        }

        Ok(EventListener::new(&self.bearer_token))
    }

    pub async fn get_device(&self, device_id: &str) -> Result<Device> {
        let response = self
            .reqwest_client
            .get(format!("https://wdm-a.wbx2.com/wdm/api/v1/devices/{}", device_id))
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
