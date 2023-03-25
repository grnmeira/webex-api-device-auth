use serde::Deserialize;

const CLIENT_ID: &str = "";
const SCOPE: &str = "spark:all";

#[derive(Deserialize,Debug)]
struct AuthorizeResponse {
    device_code: String,
    expires_in: u32,
    user_code: String,
    verification_uri: String,
    verification_uri_complete: String,
    interval: u32,
}

#[tokio::main]
async fn main() {
    let params = [("client_id", CLIENT_ID), ("scope", SCOPE)];

    let response = reqwest::Client::new()
        .post("https://webexapis.com/v1/device/authorize")
        .header(reqwest::header::CONTENT_TYPE, "application/x-www-form-urlencoded; charset=utf-8")
        .form(&params)
        .send()
        .await
        .expect("Request to authorization endpoint failed")
        .json::<AuthorizeResponse>()
        .await
        .expect("Response parsing failed");

    println!("{:#?}", response);
}