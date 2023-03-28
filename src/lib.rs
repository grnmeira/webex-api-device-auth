pub mod webex {
    pub mod auth {
        use serde::Deserialize;
        use tokio::time::{self, Duration, Instant};

        const CLIENT_ID: &str = "";
        const CLIENT_SECRET: &str = "";
        const SCOPE: &str = "spark:all";
        const GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:device_code";

        #[derive(Debug)]
        pub struct DeviceAuthenticator {
            client_id: String,
            client_secret: String
        }

        #[derive(Debug)]
        pub struct VerificationToken {
            verification_uri_complete: String,
        }

        #[derive(Debug)]
        pub enum VerificationError {
            HttpRequestError,
            ParsingError,
            GenericError
        }

        impl From<reqwest::Error> for VerificationError {
            fn from(error: reqwest::Error) -> VerificationError {
                return VerificationError::HttpRequestError;
            }
        }

        impl DeviceAuthenticator {
            pub fn new(client_id: &str, client_secret: &str) -> DeviceAuthenticator{
                return DeviceAuthenticator {
                    client_id: client_id.to_string(),
                    client_secret: client_secret.to_string()
                };
            }

            pub async fn verify(&self) -> Result<VerificationToken, VerificationError> {
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
                    //.expect("Request to authorization endpoint failed")
                let token = response.json::<AuthorizeResponse>()
                    .await?;

                return Ok(VerificationToken{ verification_uri_complete: token.verification_uri_complete });
            }
        }

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

        pub async fn authenticate_device(client_id: &str, client_secret: &str) -> Result<String, DeviceAuthenticationError>
        {
            let client = reqwest::Client::new();

            let params = [("client_id", CLIENT_ID), ("scope", SCOPE)];
            let response = client
                .post("https://webexapis.com/v1/device/authorize")
                .header(
                    reqwest::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded; charset=utf-8",
                )
                .form(&params)
                .send()
                .await
                .expect("Request to authorization endpoint failed")
                .json::<AuthorizeResponse>()
                .await
                .expect("Authorization response parsing failed");
        
            println!("{}", response.user_code);
            println!("{}", response.verification_uri);
            println!("{}", response.verification_uri_complete);
        
            let params = [
                ("grant_type", GRANT_TYPE),
                ("device_code", &response.device_code),
                ("client_id", CLIENT_ID),
            ];
        
            let mut interval = time::interval_at(
                Instant::now() + Duration::from_secs(response.interval),
                Duration::from_secs(response.interval + 1),
            );
        
            loop {
                interval.tick().await;
                let response = client
                    .post("https://webexapis.com/v1/device/token")
                    .basic_auth(CLIENT_ID, Some(CLIENT_SECRET))
                    .header(
                        reqwest::header::CONTENT_TYPE,
                        "application/x-www-form-urlencoded; charset=utf-8",
                    )
                    .form(&params)
                    .send()
                    .await
                    .expect("Request to token endpoint failed");
        
                if response.status() == 200 {
                    let token_response = response
                        .json::<TokenResponse>()
                        .await
                        .expect("Failed to parse Token Response");
                    println!("{}", token_response.access_token);
                    return Ok(token_response.access_token);
                }
        
                if response.status() != 428 {
                    return Err(DeviceAuthenticationError::HttpError(response.status().into()));
                }
            }

            return Err(DeviceAuthenticationError::UnknownError);
        }
    }
}