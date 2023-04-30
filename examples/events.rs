use clap::Parser;
use webex;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 't', help = "Bearer token")]
    bearer_token: String,
    #[arg(short = 'd', help = "Device ID")]
    device_id: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let webex_client = webex::api::Client::new(&args.bearer_token, args.device_id.as_deref());
    let mut event_listener = webex_client.listen_to_events().await.unwrap();

    loop {
        let event = event_listener.next().await.unwrap();
        println!("{:#?}", event);
    }
}
