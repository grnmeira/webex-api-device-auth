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
        println!("{:#?}", e);
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
#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(rename_all="camelCase")]
pub struct Device {
    url: Option<String>,
    device_type: Option<String>,
    name: Option<String>,
    model: Option<String>,
    localized_model: Option<String>,
    system_name: Option<String>,
    system_version: Option<String>,
    #[serde(skip_serializing)]
    websocket_url: Option<String>,
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

        return match response.error_for_status() {
            Ok(response) => {
                let json_result = response.json::<Devices>().await?;
                Ok(json_result)
            }
            Err(e) => Err(Error::from(e)),
        };
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

        let json_result = response.json::<Device>().await?;

        Ok(json_result)
    }

    pub async fn delete_device(&self, ) -> Result<()> {
        self
            .reqwest_client
            .delete("https://wdm-a.wbx2.com/wdm/api/v1/devices/")
            .bearer_auth(&self.bearer_token)
            .send()
            .await?;

        Ok(())
    }
}
