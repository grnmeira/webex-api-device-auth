use serde::Deserialize;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("HTTP error with status:")]
    HttpRequestError(#[from] reqwest::Error),
    #[error("JSON parsing error")]
    JsonParsingError,
    #[error("generic error")]
    GenericError,
}

type Result<T> = std::result::Result<T, Error>;

pub struct Client {
    bearer_token: String,
    reqwest_client: reqwest::Client,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Device {
    #[serde(rename = "webSocketUrl")]
    websocket_url: String,
}

#[derive(Deserialize, Debug)]
pub struct Devices {
    devices: Vec<Device>,
}

impl Client {
    pub fn new(bearer_token: String) -> Client {
        Client {
            bearer_token,
            reqwest_client: reqwest::Client::new(),
        }
    }

    pub async fn get_devices(&self) -> Result<Devices> {
        let response = self
            .reqwest_client
            .get("https://wdm-a.wbx2.com/wdm/api/v1/devices")
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        let json_result = response.json::<Devices>().await?;

        Ok(json_result)
    }
}
