use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use webex::{self};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 't', help = "Bearer token")]
    bearer_token: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let webex_client = webex::api::Client::new(&args.bearer_token);

    let devices = webex_client
        .get_devices()
        .await
        .expect("Error obtaining current registered devices");

    let device = devices
        .devices
        .into_iter()
        .find(|d| d.name == Some("pixoo-integration".to_string()));

    let device = if device.is_none() {
        webex_client
            .post_devices()
            .await
            .expect("Error creating device")
    } else {
        device.unwrap()
    };

    println!("{:#?}", device);

    let websocket_url = device
        .websocket_url
        .as_ref()
        .expect("No websocket URL for device");

    let request = http::Request::builder()
        .uri(websocket_url)
        .header("Authorization", format!("Bearer {}", args.bearer_token))
        .header("Sec-Websocket-Key", "APCjIuq1XI4F7MmpLXLijg==")
        .header("Sec-Websocket-Version", "13")
        .header("Connection", "Upgrade")
        .header("Host", "mercury-connection-partition2-a.wbx2.com")
        .header("Upgrade", "websocket")
        .body(())
        .unwrap();

    let (mut ws_stream, _) = connect_async(request).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    while let Some(Ok(message)) = ws_stream.next().await {
        match message {
            Message::Ping(data) => {
                let _ = ws_stream.send(Message::Pong(data)).await;
            }
            Message::Binary(_) => println!("{}", message),
            _ => (),
        }
    }
}
