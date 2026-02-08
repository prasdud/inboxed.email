pub mod gmail;
pub mod idle;
pub mod imap_client;
pub mod provider;
pub mod server_presets;
pub mod types;

pub use gmail::GmailClient;
pub use imap_client::ImapClient;
pub use types::{Email, EmailListItem, Folder, SpecialFolder};
