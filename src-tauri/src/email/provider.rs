use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::types::{Email, EmailListItem, Folder};

/// IMAP flag types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImapFlag {
    Seen,
    Flagged,
    Deleted,
    Answered,
    Draft,
}

impl ImapFlag {
    pub fn to_imap_str(&self) -> &str {
        match self {
            ImapFlag::Seen => "\\Seen",
            ImapFlag::Flagged => "\\Flagged",
            ImapFlag::Deleted => "\\Deleted",
            ImapFlag::Answered => "\\Answered",
            ImapFlag::Draft => "\\Draft",
        }
    }
}

/// Unified email provider trait — abstracts IMAP/SMTP operations
#[async_trait::async_trait]
pub trait EmailProvider: Send + Sync {
    /// List messages in a folder
    async fn list_messages(
        &self,
        folder: &str,
        max_results: u32,
        offset: u32,
    ) -> Result<Vec<EmailListItem>>;

    /// Get a single message by UID
    async fn get_message(&self, folder: &str, uid: u32) -> Result<Email>;

    /// Send an email via SMTP
    async fn send_email(
        &self,
        from: &str,
        to: Vec<String>,
        cc: Vec<String>,
        bcc: Vec<String>,
        subject: &str,
        body_html: &str,
        body_plain: &str,
    ) -> Result<()>;

    /// Set or remove flags on a message
    async fn set_flags(&self, folder: &str, uid: u32, flags: &[ImapFlag], add: bool)
        -> Result<()>;

    /// Move a message to another folder
    async fn move_message(&self, from_folder: &str, uid: u32, to_folder: &str) -> Result<()>;

    /// Delete a message permanently
    async fn delete_message(&self, folder: &str, uid: u32) -> Result<()>;

    /// List all folders/mailboxes
    async fn list_folders(&self) -> Result<Vec<Folder>>;
}
