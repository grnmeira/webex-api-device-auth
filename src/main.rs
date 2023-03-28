use clap::Parser;
use webex_api_device_auth::webex;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'i', help = "client (integration) ID")]
    client_id: String,
    #[arg(short = 's', help = "client (integration) secret")]
    client_secret: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let authenticator = webex::auth::DeviceAuthenticator::new(&args.client_id, &args.client_secret);
    let verification_token = authenticator.verify().await.unwrap_or_else(|error| {
        panic!("Error obtaining verification token: {:#?}", error);
    });
    println!("{:#?}", verification_token);
    let bearer_token = authenticator
        .wait_for_authentication(&verification_token)
        .await
        .unwrap_or_else(|error| {
            panic!("Error obtaining bearer token: {:#?}", error);
        });
    println!("Bearer token:");
    println!("{}", bearer_token);
}
