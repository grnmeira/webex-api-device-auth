use chrono::serde::ts_milliseconds_option;
use chrono::{DateTime, Utc};
use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use webex::{self};

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "eventType")]
pub enum Data {
    #[serde(rename = "apheleia.subscription_update")]
    SubscriptionUpdate {
        subject: Option<String>,
        category: Option<String>,
        status: Option<String>
    },
    #[serde(rename = "conversation.activity")]
    ConversationActivity {
        id: String
    }
}

#[allow(dead_code)]
#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: Option<String>,
    pub data: Option<Data>,
    pub filter_message: Option<bool>,
    //pub headers: Option<String>,
    pub sequence_number: Option<u32>,
    #[serde(with = "ts_milliseconds_option")]
    pub timestamp: Option<DateTime<Utc>>,
    pub tracking_id: Option<String>,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 't', help = "Bearer token")]
    bearer_token: String,
    #[arg(short = 'd', help = "Device ID")]
    device_id: Option<String>
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
            Message::Binary(data) => {
                println!("{:#?}", &String::from_utf8_lossy(&data));
                let e = serde_json::from_str::<Event>(
                    &String::from_utf8(data).expect("Error decoding UT8"),
                );
                println!("{:#?}", e);
            }
            _ => (),
        }
    }
}
