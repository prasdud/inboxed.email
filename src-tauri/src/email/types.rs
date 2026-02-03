use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Email {
    pub id: String,
    pub thread_id: String,
    pub subject: String,
    pub from: String,
    pub from_email: String,
    pub to: Vec<String>,
    pub date: String,
    pub snippet: String,
    pub body_html: Option<String>,
    pub body_plain: Option<String>,
    pub labels: Vec<String>,
    pub is_read: bool,
    pub is_starred: bool,
    pub has_attachments: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailListItem {
    pub id: String,
    pub thread_id: String,
    pub subject: String,
    pub from: String,
    pub from_email: String,
    pub date: String,
    pub snippet: String,
    pub is_read: bool,
    pub is_starred: bool,
    pub has_attachments: bool,
}

#[derive(Debug, Deserialize)]
pub struct GmailMessage {
    pub id: String,
    #[serde(rename = "threadId")]
    pub thread_id: String,
    #[serde(rename = "labelIds", default)]
    pub label_ids: Vec<String>,
    pub snippet: String,
    pub payload: Option<MessagePayload>,
    #[serde(rename = "internalDate")]
    pub internal_date: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessagePayload {
    pub headers: Vec<MessageHeader>,
    pub body: Option<MessageBody>,
    pub parts: Option<Vec<MessagePart>>,
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageHeader {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessageBody {
    pub data: Option<String>,
    pub size: Option<u64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MessagePart {
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
    pub body: Option<MessageBody>,
    pub parts: Option<Vec<MessagePart>>,
    pub filename: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct GmailListResponse {
    pub messages: Option<Vec<GmailMessageId>>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
    #[serde(rename = "resultSizeEstimate")]
    pub result_size_estimate: Option<u64>,
}

#[derive(Debug, Deserialize)]
pub struct GmailMessageId {
    pub id: String,
    #[serde(rename = "threadId")]
    pub thread_id: String,
}
