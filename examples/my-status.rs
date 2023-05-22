use clap::Parser;
use webex;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 't', help = "Bearer token")]
    bearer_token: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let webex_client = webex::api::Client::new(&args.bearer_token, None);

    let me = webex_client.get_my_own_details().await.expect("not able to get my own details");

    println!("{:#?}", me);
}