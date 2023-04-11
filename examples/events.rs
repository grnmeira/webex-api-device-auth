use clap::Parser;
use webex::{self};
use tokio_tungstenite as tungstenite;
use futures_util::{StreamExt};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 't', help = "Bearer token")]
    bearer_token: String,
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

    println!("{:#?}", device);

    let websocket_url = device.websocket_url.as_ref().expect("No websocket URL for device");

    let url = url::Url::parse(&websocket_url).unwrap();
    let (ws_stream, _) = tungstenite::connect_async_tls_with_config(url, None, None).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (_, ws_stream) = ws_stream.split();
    ws_stream.for_each(|message| async move {
        println!("{:#?}", message);
    }).await;

    //let device = webex_client.post_devices().await.expect("Error creating device");
    //let devices = webex_client.get_devices().await.expect("Error requesting devices");

    // for device in devices.devices.iter() {
    //     if device.device_type == Some("UNKNOWN".to_string()) {
    //         if let Some(device_url) = &device.url {
    //             webex_client.delete_device(&device_url).await.expect("Failure deleting device");
    //         }
    //     }
    // }
}