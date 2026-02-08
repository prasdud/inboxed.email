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
    pub date_timestamp: i64,
    pub snippet: String,
    pub body_html: Option<String>,
    pub body_plain: Option<String>,
    pub labels: Vec<String>,
    pub is_read: bool,
    pub is_starred: bool,
    pub has_attachments: bool,
    // IMAP-specific fields
    pub account_id: String,
    pub uid: u32,
    pub folder: String,
    pub message_id: String,
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

/// Represents an IMAP folder/mailbox
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    /// Full folder path (e.g., "[Gmail]/Sent Mail")
    pub name: String,
    /// Display name (e.g., "Sent Mail")
    pub display_name: String,
    /// Special folder type, if detected
    pub special: Option<SpecialFolder>,
    /// Hierarchy delimiter (e.g., "/")
    pub delimiter: Option<String>,
}

/// Well-known special folder types (RFC 6154)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SpecialFolder {
    Inbox,
    Sent,
    Trash,
    Drafts,
    Spam,
    Archive,
    Starred,
}

/// Parsed email address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailAddress {
    pub name: Option<String>,
    pub address: String,
}

impl EmailAddress {
    pub fn display(&self) -> String {
        match &self.name {
            Some(name) => format!("{} <{}>", name, self.address),
            None => self.address.clone(),
        }
    }
}
