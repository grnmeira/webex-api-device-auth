use clap::Parser;
use serde::Deserialize;
use webex::{self};

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
struct Devices {
    devices: Vec<Device>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let webex_client = webex::api::Client::new(args.bearer_token);

    let devices = webex_client.get_devices().await.expect("Error obtaining current registered devices");

    let device = devices.devices.into_iter().find(|d| d.name == Some("pixoo-integration".to_string()));

    let device = if device.is_none() {
        webex_client.post_devices().await.expect("Error creating device")
    } else {
        device.unwrap()
    };

    //let device = webex_client.post_devices().await.expect("Error creating device");
    //let devices = webex_client.get_devices().await.expect("Error requesting devices");

    // for device in devices.devices.iter() {
    //     if device.device_type == Some("UNKNOWN".to_string()) {
    //         if let Some(device_url) = &device.url {
    //             webex_client.delete_device(&device_url).await.expect("Failure deleting device");
    //         }
    //     }
    // }

    println!("{:#?}", device);
}