use tokio::time::{self, Duration, Instant};
use webex_api_device_auth::webex;

const CLIENT_ID: &str = "";
const CLIENT_SECRET: &str = "";

#[tokio::main]
async fn main() {
    //webex::auth::authenticate_device(CLIENT_ID, CLIENT_SECRET).await.expect("Failure anthenticating device");
    let authenticator = webex::auth::DeviceAuthenticator::new(CLIENT_ID, CLIENT_SECRET);
    let verification_token = authenticator.verify().await.unwrap_or_else(|error| {
        panic!("Error obtaining verification token: {:#?}", error);
    });
    println!("{:#?}", verification_token);
    /*
    println!("{}", &verification_token.verification_uri);
    let verification_result = verification_token.wait_for_result().await;
    println!("{}", verification_result.access_token);

    // renew token
    // expire token
    */
}
