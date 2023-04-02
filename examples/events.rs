use clap::Parser;
use serde::Deserialize;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 't', help = "Bearer token")]
    bearer_token: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct Device {
    #[serde(rename="webSocketUrl")]
    websocket_url: String,
}

#[derive(Deserialize, Debug)]
struct DevicesResponse {
    devices: Vec<Device>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let client = reqwest::Client::new();
    let response = client.get("https://wdm-a.wbx2.com/wdm/api/v1/devices")
        .bearer_auth(args.bearer_token)
        .send()
        .await.expect("Error while getting devices");

    println!("{:#?}", response);

    let devices_response = response.json::<DevicesResponse>().await.expect("Error parsing devices");

    println!("{:#?}", devices_response);
}