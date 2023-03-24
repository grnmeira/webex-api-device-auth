const CLIENT_ID: &str = "";
const SCOPE: &str = "";

#[tokio::main]
async fn main() {
    let response = reqwest::Client::new()
        .post("https://webexapis.com/v1/device/authorize")
        .header(reqwest::header::CONTENT_TYPE, "application/x-www-form-urlencoded; charset=utf-8")
        // this is not properly encoded
        .body(format!("client_id={}&scope={}", CLIENT_ID, SCOPE))
        .send()
        .await
        .expect("send")
        .text()
        .await;

    println!("{:#?}", response);
}