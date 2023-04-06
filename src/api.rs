use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("HTTP error with status: {0}")]
    HttpStatus(u16),
    #[error("JSON parsing error: {0}")]
    JsonParsingError(String),
    #[error("generic error")]
    GenericError,
}

type Result<T> = std::result::Result<T, Error>;

impl From<reqwest::Error> for Error {
    fn from(e: reqwest::Error) -> Self {
        if let Some(status_code) = e.status() {
            Error::HttpStatus(status_code.as_u16())
        } else {
            Error::GenericError
        }
    }
}

pub struct Client {
    bearer_token: String,
    reqwest_client: reqwest::Client,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Device {
    #[serde(rename = "webSocketUrl")]
    websocket_url: String,
}

#[derive(Deserialize, Debug)]
pub struct Devices {
    devices: Vec<Device>,
}

#[derive(Serialize, Debug)]
pub struct DeviceRequest {
    #[serde(rename = "deviceType")]
    device_type: String,
    name: String,
    model: String,
    #[serde(rename = "localizedModel")]
    localized_model: String,
    #[serde(rename = "systemName")]
    system_name: String,
    #[serde(rename = "systemVersion")]
    system_version: String,
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

        return match response.error_for_status() {
            Ok(response) => {
                let json_result = response.json::<Devices>().await?;
                Ok(json_result)
            }
            Err(e) => Err(Error::from(e)),
        };
    }

    pub async fn post_devices(&self) -> Result<Device> {
        let device_object = DeviceRequest {
            device_type: "pixoo".to_string(),
            name: "pixoo-integration".to_string(),
            model: "pixoo-64".to_string(),
            localized_model: "".to_string(),
            system_name: "Windows".to_string(),
            system_version: "10".to_string(),
        };
        let response = self
            .reqwest_client
            .post("https://wdm-a.wbx2.com/wdm/api/v1/devices")
            .json(&device_object)
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        let json_result = response.json::<Device>().await?;

        Ok(json_result)
    }
}
