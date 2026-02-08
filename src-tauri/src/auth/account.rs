use serde::{Deserialize, Serialize};

use crate::email::server_presets::{AuthType, ProviderType};

/// Represents a connected email account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub email: String,
    pub display_name: String,
    pub provider: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub auth_type: String,
    pub is_active: bool,
    pub created_at: i64,
    pub last_synced_at: Option<i64>,
}

impl Account {
    pub fn new(
        email: String,
        display_name: String,
        provider: ProviderType,
        imap_host: String,
        imap_port: u16,
        smtp_host: String,
        smtp_port: u16,
        auth_type: AuthType,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            email,
            display_name,
            provider: provider.as_str().to_string(),
            imap_host,
            imap_port,
            smtp_host,
            smtp_port,
            auth_type: match auth_type {
                AuthType::OAuth2 => "oauth2".to_string(),
                AuthType::Password => "password".to_string(),
            },
            is_active: true,
            created_at: chrono::Utc::now().timestamp(),
            last_synced_at: None,
        }
    }

    pub fn provider_type(&self) -> ProviderType {
        ProviderType::from_str(&self.provider)
    }

    pub fn auth_type_enum(&self) -> AuthType {
        match self.auth_type.as_str() {
            "oauth2" => AuthType::OAuth2,
            _ => AuthType::Password,
        }
    }
}
