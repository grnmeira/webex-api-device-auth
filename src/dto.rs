use chrono::serde::ts_milliseconds_option;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct WebexError {
    pub code: Option<String>,
    pub reason: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Serialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub url: Option<String>,
    pub device_type: Option<String>,
    pub name: Option<String>,
    pub model: Option<String>,
    pub localized_model: Option<String>,
    pub system_name: Option<String>,
    pub system_version: Option<String>,
    #[serde(skip_serializing, rename = "webSocketUrl")]
    pub websocket_url: Option<String>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
pub struct Devices {
    pub devices: Vec<Device>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub id: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "eventType")]
pub enum Data {
    #[serde(rename = "apheleia.subscription_update")]
    SubscriptionUpdate {
        subject: Option<String>,
        category: Option<String>,
        status: Option<String>,
    },
    #[serde(rename = "conversation.activity")]
    ConversationActivity { activity: Activity },
    #[serde(other)]
    UnkownData,
}

#[allow(dead_code)]
#[derive(Deserialize, Default, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    pub id: Option<String>,
    pub data: Option<Data>,
    pub filter_message: Option<bool>,
    pub sequence_number: Option<u32>,
    #[serde(with = "ts_milliseconds_option")]
    pub timestamp: Option<DateTime<Utc>>,
    pub tracking_id: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    pub id: String,
    pub status: String,
}
