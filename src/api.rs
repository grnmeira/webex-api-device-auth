use crate::dto;
use crate::error::{Error, Result};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    connect_async, tungstenite, tungstenite::protocol::Message, MaybeTlsStream, WebSocketStream,
};

type WebsocketStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

pub struct EventListener {
    stream: WebsocketStream,
}

impl EventListener {
    fn new(stream: WebsocketStream) -> EventListener {
        EventListener { stream }
    }

    pub async fn next(&mut self) -> Result<dto::Event> {
        while let Some(Ok(message)) = self.stream.next().await {
            match message {
                Message::Ping(data) => {
                    self.stream.send(Message::Pong(data)).await?;
                }
                Message::Binary(data) => {
                    return String::from_utf8(data)
                        .map_err(|err| Error::JsonParsingError(err.to_string()))
                        .and_then(|string| {
                            serde_json::from_str::<dto::Event>(&string)
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

    pub async fn get_device(&self, device_id: &str) -> Result<dto::Device> {
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
            let json_result = response.json::<dto::Device>().await?;
            return Ok(json_result);
        }

        match response.status().as_u16() {
            code @ 400..=499 => Err(Error::HttpStatus(code, None)),
            error_code => Err(Error::HttpStatus(error_code, None)),
        }
    }

    pub async fn get_devices(&self) -> Result<dto::Devices> {
        let response = self
            .reqwest_client
            .get("https://wdm-a.wbx2.com/wdm/api/v1/devices")
            .header("Accept", "application/json")
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        if response.status().is_success() {
            let json_result = response.json::<dto::Devices>().await?;
            return Ok(json_result);
        }

        match response.status().as_u16() {
            code @ 400..=499 => Err(Error::HttpStatus(code, None)),
            error_code => Err(Error::HttpStatus(error_code, None)),
        }
    }

    pub async fn post_devices(&self) -> Result<dto::Device> {
        let device_object = dto::Device {
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
            let json_result = response.json::<dto::Device>().await?;
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

    pub async fn get_my_own_details(&self) -> Result<dto::Person> {
        let response = self
            .reqwest_client
            .get("https://webexapis.com/v1/people/me")
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        if response.status().is_success() {
            let json_result = response.json::<dto::Person>().await?;
            return Ok(json_result);
        }

        match response.status().as_u16() {
            code @ 400..=499 => Err(Error::HttpStatus(code, None)),
            error_code => Err(Error::HttpStatus(error_code, None)),
        }
    }
}
