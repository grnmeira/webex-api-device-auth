use serde::Deserialize;
use tokio::time::{self, Duration, Instant};

const SCOPE: &str = "spark:all";
const GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:device_code";

#[derive(Debug)]
pub struct DeviceAuthenticator {
    client_id: String,
    client_secret: String,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct VerificationToken {
    code: String,
    verification_uri: String,
    verification_uri_complete: String,
    polling_interval: u64,
}

#[derive(Debug)]
pub enum Error {
    HttpRequestError,
    ParsingError,
    GenericError,
}

// This can be easily done with the "thiserror" crate,
// but I'll have some fun as I'm not fully familiar with
// the language yet.
impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Error {
        if error.is_status() {
            Error::HttpRequestError
        } else if error.is_decode() {
            Error::ParsingError
        } else {
            Error::GenericError
        }
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct AuthorizeResponse {
    device_code: String,
    expires_in: u64,
    user_code: String,
    verification_uri: String,
    verification_uri_complete: String,
    interval: u64,
}

#[derive(Deserialize, Debug)]
struct TokenResponse {
    access_token: String,
}

#[derive(Debug)]
pub enum DeviceAuthenticationError {
    HttpError(u16),
    UnknownError,
}

impl DeviceAuthenticator {
    pub fn new(client_id: &str, client_secret: &str) -> DeviceAuthenticator {
        DeviceAuthenticator {
            client_id: client_id.to_string(),
            client_secret: client_secret.to_string(),
        }
    }

    pub async fn verify(&self) -> Result<VerificationToken, Error> {
        let client = reqwest::Client::new();
        let params = [("client_id", self.client_id.as_str()), ("scope", SCOPE)];
        let response = client
            .post("https://webexapis.com/v1/device/authorize")
            .header(
                reqwest::header::CONTENT_TYPE,
                "application/x-www-form-urlencoded; charset=utf-8",
            )
            .form(&params)
            .send()
            .await?;

        let token = response.json::<AuthorizeResponse>().await?;

        Ok(VerificationToken {
            code: token.device_code,
            verification_uri: token.verification_uri,
            verification_uri_complete: token.verification_uri_complete,
            polling_interval: token.interval,
        })
    }

    pub async fn wait_for_authentication(
        &self,
        verification_token: &VerificationToken,
    ) -> Result<String, Error> {
        let client = reqwest::Client::new();

        let params = [
            ("grant_type", GRANT_TYPE),
            ("device_code", &verification_token.code),
            ("client_id", &self.client_id),
        ];

        let mut interval = time::interval_at(
            Instant::now() + Duration::from_secs(verification_token.polling_interval),
            Duration::from_secs(verification_token.polling_interval + 1),
        );

        loop {
            interval.tick().await;
            let response = client
                .post("https://webexapis.com/v1/device/token")
                .basic_auth(&self.client_id, Some(&self.client_secret))
                .header(
                    reqwest::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded; charset=utf-8",
                )
                .form(&params)
                .send()
                .await?;

            if response.status() == 200 {
                let token_response = response.json::<TokenResponse>().await?;
                return Ok(token_response.access_token);
            }

            if response.status() != 428 {
                return Err(Error::HttpRequestError);
            }
        }
    }
}
