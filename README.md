# Webex API Device Authentication

This is a proof of concept for authentication with the Webex Developer API using the "device grant authentication flow".

This flow works really well for Webex Integrations made for devices that can be a bit awkward to authenticate with the Webex API, such as stand alone displays. For more details about this flow: https://developer.webex.com/docs/login-with-webex#device-grant-flow.

This repository is not recommended for use, this work has been ported to the `webex-rust` crate (https://crates.io/crates/webex), if you're interested in this authentication flow.
