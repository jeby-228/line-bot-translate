use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LinePayload {
    pub events: Vec<Event>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    #[serde(rename = "type")]
    pub event_type: String,
    pub reply_token: Option<String>,
    pub message: Option<Message>,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    #[serde(rename = "type")]
    pub message_type: String,
    pub text: Option<String>,
}
