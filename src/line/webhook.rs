use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LinePayload {
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Event {
    #[serde(rename = "type")]
    pub event_type: String,
    pub reply_token: Option<String>,
    pub source: Option<EventSource>,
    pub message: Option<Message>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EventSource {
    #[serde(rename = "type")]
    pub source_type: Option<String>,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Message {
    pub id: Option<String>,
    #[serde(rename = "type")]
    pub message_type: String,
    pub text: Option<String>,
}
