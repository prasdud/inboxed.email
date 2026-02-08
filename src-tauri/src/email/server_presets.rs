use serde::{Deserialize, Serialize};

/// Authentication type for an email account
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthType {
    /// OAuth2 XOAUTH2 SASL mechanism
    OAuth2,
    /// Plain password / app password
    Password,
}

/// Email provider type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProviderType {
    Gmail,
    Outlook,
    Yahoo,
    Custom,
}

impl ProviderType {
    pub fn as_str(&self) -> &str {
        match self {
            ProviderType::Gmail => "gmail",
            ProviderType::Outlook => "outlook",
            ProviderType::Yahoo => "yahoo",
            ProviderType::Custom => "custom",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "gmail" => ProviderType::Gmail,
            "outlook" | "microsoft" | "hotmail" => ProviderType::Outlook,
            "yahoo" => ProviderType::Yahoo,
            _ => ProviderType::Custom,
        }
    }
}

/// Server configuration for IMAP and SMTP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub imap_host: String,
    pub imap_port: u16,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub use_tls: bool,
}

/// Well-known server presets
pub fn get_server_preset(provider: &ProviderType) -> Option<ServerConfig> {
    match provider {
        ProviderType::Gmail => Some(ServerConfig {
            imap_host: "imap.gmail.com".to_string(),
            imap_port: 993,
            smtp_host: "smtp.gmail.com".to_string(),
            smtp_port: 465,
            use_tls: true,
        }),
        ProviderType::Outlook => Some(ServerConfig {
            imap_host: "outlook.office365.com".to_string(),
            imap_port: 993,
            smtp_host: "smtp.office365.com".to_string(),
            smtp_port: 587,
            use_tls: true,
        }),
        ProviderType::Yahoo => Some(ServerConfig {
            imap_host: "imap.mail.yahoo.com".to_string(),
            imap_port: 993,
            smtp_host: "smtp.mail.yahoo.com".to_string(),
            smtp_port: 465,
            use_tls: true,
        }),
        ProviderType::Custom => None,
    }
}

/// Detect provider from email domain
pub fn detect_provider(email: &str) -> ProviderType {
    let domain = email
        .split('@')
        .nth(1)
        .unwrap_or("")
        .to_lowercase();

    match domain.as_str() {
        "gmail.com" | "googlemail.com" => ProviderType::Gmail,
        "outlook.com" | "hotmail.com" | "live.com" | "msn.com" => ProviderType::Outlook,
        "yahoo.com" | "ymail.com" | "rocketmail.com" => ProviderType::Yahoo,
        _ => ProviderType::Custom,
    }
}

/// Get default auth type for a provider
pub fn default_auth_type(provider: &ProviderType) -> AuthType {
    match provider {
        ProviderType::Gmail | ProviderType::Outlook => AuthType::OAuth2,
        ProviderType::Yahoo | ProviderType::Custom => AuthType::Password,
    }
}

/// Special folder names vary across providers
pub struct SpecialFolders {
    pub sent: &'static str,
    pub trash: &'static str,
    pub drafts: &'static str,
    pub spam: &'static str,
    pub archive: &'static str,
}

pub fn get_special_folders(provider: &ProviderType) -> SpecialFolders {
    match provider {
        ProviderType::Gmail => SpecialFolders {
            sent: "[Gmail]/Sent Mail",
            trash: "[Gmail]/Trash",
            drafts: "[Gmail]/Drafts",
            spam: "[Gmail]/Spam",
            archive: "[Gmail]/All Mail",
        },
        ProviderType::Outlook => SpecialFolders {
            sent: "Sent",
            trash: "Deleted",
            drafts: "Drafts",
            spam: "Junk",
            archive: "Archive",
        },
        _ => SpecialFolders {
            sent: "Sent",
            trash: "Trash",
            drafts: "Drafts",
            spam: "Spam",
            archive: "Archive",
        },
    }
}
