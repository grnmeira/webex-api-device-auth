use serde::Deserialize;
use tokio::time::{self, Duration, Instant};

const CLIENT_ID: &str = "";
const CLIENT_SECRET: &str = "";
const SCOPE: &str = "spark:all";
const GRANT_TYPE: &str = "urn:ietf:params:oauth:grant-type:device_code";

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

#[tokio::main]
async fn main() {
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
            let token_response = &response
                .json::<TokenResponse>()
                .await
                .expect("Failed to parse Token Response");
            println!("{}", token_response.access_token);
            break;
        }

        if response.status() != 428 {
            break;
        }
    }
}
