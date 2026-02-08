use anyhow::{Context, Result};
use async_imap::extensions::idle::IdleResponse;
use async_imap::types::{Fetch, Flag};
use async_native_tls::TlsConnector;
use futures::StreamExt;
use lettre::message::{header::ContentType, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use mail_parser::MessageParser;
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_util::compat::TokioAsyncReadCompatExt;

use super::provider::{EmailProvider, ImapFlag};
use super::server_presets::{AuthType, ProviderType, ServerConfig};
use super::types::{Email, EmailListItem, Folder, SpecialFolder};

/// Type alias for the TLS stream using tokio compat
type ImapTlsStream = async_native_tls::TlsStream<tokio_util::compat::Compat<TcpStream>>;
type ImapSession = async_imap::Session<ImapTlsStream>;

/// Credentials for connecting to IMAP/SMTP
#[derive(Debug, Clone)]
pub enum ImapCredentials {
    OAuth2 {
        user: String,
        access_token: String,
    },
    Password {
        user: String,
        password: String,
    },
}

impl ImapCredentials {
    pub fn user(&self) -> &str {
        match self {
            ImapCredentials::OAuth2 { user, .. } => user,
            ImapCredentials::Password { user, .. } => user,
        }
    }
}

/// IMAP/SMTP client for a single email account
pub struct ImapClient {
    pub account_id: String,
    pub email: String,
    pub provider: ProviderType,
    pub server_config: ServerConfig,
    credentials: ImapCredentials,
    session: Arc<Mutex<Option<ImapSession>>>,
}

impl ImapClient {
    pub fn new(
        account_id: String,
        email: String,
        provider: ProviderType,
        server_config: ServerConfig,
        credentials: ImapCredentials,
    ) -> Self {
        Self {
            account_id,
            email,
            provider,
            server_config,
            credentials,
            session: Arc::new(Mutex::new(None)),
        }
    }

    pub fn update_credentials(&mut self, credentials: ImapCredentials) {
        self.credentials = credentials;
    }

    /// Connect to IMAP server and authenticate
    async fn connect(&self) -> Result<ImapSession> {
        let tls = TlsConnector::new();
        let tcp = TcpStream::connect((
            self.server_config.imap_host.as_str(),
            self.server_config.imap_port,
        ))
        .await
        .context("Failed to connect to IMAP server")?;

        // Convert tokio TcpStream to futures_io compatible stream
        let tcp_compat = tcp.compat();

        let tls_stream = tls
            .connect(&self.server_config.imap_host, tcp_compat)
            .await
            .context("TLS handshake failed")?;

        let client = async_imap::Client::new(tls_stream);

        let session = match &self.credentials {
            ImapCredentials::OAuth2 { user, access_token } => {
                let auth_string = format!(
                    "user={}\x01auth=Bearer {}\x01\x01",
                    user, access_token
                );
                client
                    .authenticate("XOAUTH2", XOAuth2Authenticator(auth_string))
                    .await
                    .map_err(|(e, _)| anyhow::anyhow!("XOAUTH2 authentication failed: {}", e))?
            }
            ImapCredentials::Password { user, password } => client
                .login(user, password)
                .await
                .map_err(|(e, _)| anyhow::anyhow!("IMAP login failed: {}", e))?,
        };

        Ok(session)
    }

    async fn get_session(&self) -> Result<tokio::sync::MutexGuard<'_, Option<ImapSession>>> {
        let mut guard = self.session.lock().await;
        if guard.is_none() {
            let session = self.connect().await?;
            *guard = Some(session);
        }
        Ok(guard)
    }

    pub async fn reconnect(&self) -> Result<()> {
        let mut guard = self.session.lock().await;
        if let Some(mut session) = guard.take() {
            let _ = session.logout().await;
        }
        let session = self.connect().await?;
        *guard = Some(session);
        Ok(())
    }

    /// Parse a raw email message into our Email type
    pub fn parse_raw_email(
        &self,
        uid: u32,
        folder: &str,
        raw: &[u8],
        flags: &[Flag<'_>],
    ) -> Result<Email> {
        let parsed = MessageParser::default()
            .parse(raw)
            .context("Failed to parse email message")?;

        let subject = parsed
            .subject()
            .unwrap_or("(No Subject)")
            .to_string();

        let from = parsed
            .from()
            .and_then(|addrs| addrs.first())
            .map(|addr| {
                if let Some(name) = addr.name() {
                    format!("{} <{}>", name, addr.address().unwrap_or(""))
                } else {
                    addr.address().unwrap_or("").to_string()
                }
            })
            .unwrap_or_else(|| "Unknown".to_string());

        let from_email = parsed
            .from()
            .and_then(|addrs| addrs.first())
            .and_then(|addr| addr.address())
            .unwrap_or("")
            .to_string();

        let to: Vec<String> = parsed
            .to()
            .map(|addrs| {
                addrs
                    .iter()
                    .map(|addr| {
                        if let Some(name) = addr.name() {
                            format!("{} <{}>", name, addr.address().unwrap_or(""))
                        } else {
                            addr.address().unwrap_or("").to_string()
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        let date = parsed
            .date()
            .map(|d| d.to_rfc3339())
            .unwrap_or_default();

        let date_timestamp = parsed
            .date()
            .map(|d| d.to_timestamp())
            .unwrap_or_else(|| chrono::Utc::now().timestamp());

        let body_html = parsed.body_html(0).map(|s| s.to_string());
        let body_plain = parsed.body_text(0).map(|s| s.to_string());

        let snippet = body_plain
            .as_deref()
            .unwrap_or("")
            .chars()
            .take(200)
            .collect::<String>()
            .replace('\n', " ")
            .replace('\r', "");

        let is_read = flags.iter().any(|f| matches!(f, Flag::Seen));
        let is_starred = flags.iter().any(|f| matches!(f, Flag::Flagged));
        let has_attachments = parsed.attachment_count() > 0;

        let message_id = parsed.message_id().unwrap_or("").to_string();
        let thread_id = self.compute_thread_id(&parsed);
        let id = format!("{}:{}:{}", self.account_id, folder, uid);

        let mut labels = Vec::new();
        if !is_read {
            labels.push("UNREAD".to_string());
        }
        if is_starred {
            labels.push("STARRED".to_string());
        }
        if folder.eq_ignore_ascii_case("INBOX") {
            labels.push("INBOX".to_string());
        }

        Ok(Email {
            id,
            thread_id,
            subject,
            from,
            from_email,
            to,
            date,
            date_timestamp,
            snippet,
            body_html,
            body_plain,
            labels,
            is_read,
            is_starred,
            has_attachments,
            account_id: self.account_id.clone(),
            uid,
            folder: folder.to_string(),
            message_id,
        })
    }

    fn compute_thread_id(&self, parsed: &mail_parser::Message<'_>) -> String {
        // Try In-Reply-To first for threading
        // in_reply_to() returns &HeaderValue directly in mail-parser 0.9
        let irt = parsed.in_reply_to();
        if let Some(text) = irt.as_text() {
            if !text.is_empty() {
                return format!("{:x}", md5::compute(text.as_bytes()));
            }
        }

        // Try References header
        let refs = parsed.references();
        if let Some(text) = refs.as_text() {
            if let Some(first) = text.split_whitespace().next() {
                if !first.is_empty() {
                    return format!("{:x}", md5::compute(first.as_bytes()));
                }
            }
        }

        // Fallback to own message-id
        if let Some(mid) = parsed.message_id() {
            if !mid.is_empty() {
                return format!("{:x}", md5::compute(mid.as_bytes()));
            }
        }

        uuid::Uuid::new_v4().to_string()
    }

    pub fn to_list_item(email: &Email) -> EmailListItem {
        EmailListItem {
            id: email.id.clone(),
            thread_id: email.thread_id.clone(),
            subject: email.subject.clone(),
            from: email.from.clone(),
            from_email: email.from_email.clone(),
            date: email.date.clone(),
            snippet: email.snippet.clone(),
            is_read: email.is_read,
            is_starred: email.is_starred,
            has_attachments: email.has_attachments,
        }
    }

    async fn build_smtp_transport(&self) -> Result<AsyncSmtpTransport<Tokio1Executor>> {
        let builder = if self.server_config.smtp_port == 465 {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&self.server_config.smtp_host)?
                .port(self.server_config.smtp_port)
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&self.server_config.smtp_host)?
                .port(self.server_config.smtp_port)
        };

        let transport = match &self.credentials {
            ImapCredentials::OAuth2 { user, access_token } => builder
                .credentials(Credentials::new(user.clone(), access_token.clone()))
                .authentication(vec![Mechanism::Xoauth2])
                .build(),
            ImapCredentials::Password { user, password } => builder
                .credentials(Credentials::new(user.clone(), password.clone()))
                .build(),
        };

        Ok(transport)
    }

    pub async fn idle_wait(&self, folder: &str, timeout_secs: u64) -> Result<bool> {
        let mut guard = self.session.lock().await;
        let session = guard.take().context("No IMAP session")?;

        // Select folder first, then start IDLE
        let mut session = session;
        session
            .select(folder)
            .await
            .context("Failed to select folder")?;

        let mut idle = session.idle();
        idle.init().await.context("Failed to init IDLE")?;

        let (idle_wait, _stop) =
            idle.wait_with_timeout(std::time::Duration::from_secs(timeout_secs));
        let result = idle_wait.await.context("IDLE wait failed")?;

        let new_mail = match result {
            IdleResponse::NewData(_) => true,
            IdleResponse::Timeout => false,
            IdleResponse::ManualInterrupt => false,
        };

        // Get session back from idle handle
        let session = idle.done().await.context("Failed to finish IDLE")?;
        *guard = Some(session);

        Ok(new_mail)
    }

    /// Parse a FETCH response into an EmailListItem
    fn parse_fetch_to_list_item(&self, uid: u32, folder: &str, fetch: &Fetch) -> EmailListItem {
        let flags: Vec<Flag<'_>> = fetch.flags().collect();
        let is_read = flags.iter().any(|f| matches!(f, Flag::Seen));
        let is_starred = flags.iter().any(|f| matches!(f, Flag::Flagged));

        let (subject, from, from_email, date) = if let Some(envelope) = fetch.envelope() {
            let subject = envelope
                .subject
                .as_ref()
                .and_then(|s| std::str::from_utf8(s).ok())
                .unwrap_or("(No Subject)")
                .to_string();

            let (from, from_email) = envelope
                .from
                .as_ref()
                .and_then(|addrs| addrs.first())
                .map(|addr| {
                    let name = addr
                        .name
                        .as_ref()
                        .and_then(|n| std::str::from_utf8(n).ok())
                        .unwrap_or("");
                    let mailbox = addr
                        .mailbox
                        .as_ref()
                        .and_then(|m| std::str::from_utf8(m).ok())
                        .unwrap_or("");
                    let host = addr
                        .host
                        .as_ref()
                        .and_then(|h| std::str::from_utf8(h).ok())
                        .unwrap_or("");
                    let email = format!("{}@{}", mailbox, host);
                    if name.is_empty() {
                        (email.clone(), email)
                    } else {
                        (format!("{} <{}>", name, email), email)
                    }
                })
                .unwrap_or_else(|| ("Unknown".to_string(), String::new()));

            let date = envelope
                .date
                .as_ref()
                .and_then(|d| std::str::from_utf8(d).ok())
                .unwrap_or("")
                .to_string();

            (subject, from, from_email, date)
        } else {
            (
                "(No Subject)".to_string(),
                "Unknown".to_string(),
                String::new(),
                String::new(),
            )
        };

        let id = format!("{}:{}:{}", self.account_id, folder, uid);

        EmailListItem {
            id,
            thread_id: String::new(),
            subject,
            from,
            from_email,
            date,
            snippet: String::new(),
            is_read,
            is_starred,
            has_attachments: false,
        }
    }

    fn detect_special_folder(
        &self,
        name: &str,
        _attributes: &[async_imap::types::NameAttribute<'_>],
    ) -> Option<SpecialFolder> {
        // Name-based detection (works across all IMAP servers)
        let lower = name.to_lowercase();
        if lower == "inbox" {
            Some(SpecialFolder::Inbox)
        } else if lower.contains("sent") {
            Some(SpecialFolder::Sent)
        } else if lower.contains("trash") || lower.contains("deleted") {
            Some(SpecialFolder::Trash)
        } else if lower.contains("draft") {
            Some(SpecialFolder::Drafts)
        } else if lower.contains("spam") || lower.contains("junk") {
            Some(SpecialFolder::Spam)
        } else if lower.contains("archive") || lower.contains("all mail") {
            Some(SpecialFolder::Archive)
        } else {
            None
        }
    }
}

/// XOAUTH2 authenticator for async-imap
struct XOAuth2Authenticator(String);

impl async_imap::Authenticator for XOAuth2Authenticator {
    type Response = String;

    fn process(&mut self, _data: &[u8]) -> Self::Response {
        self.0.clone()
    }
}

#[async_trait::async_trait]
impl EmailProvider for ImapClient {
    async fn list_messages(
        &self,
        folder: &str,
        max_results: u32,
        offset: u32,
    ) -> Result<Vec<EmailListItem>> {
        let mut guard = self.get_session().await?;
        let session = guard.as_mut().context("No IMAP session")?;

        let mailbox = session
            .select(folder)
            .await
            .context("Failed to select folder")?;

        let total = mailbox.exists;
        if total == 0 {
            return Ok(vec![]);
        }

        let end = total.saturating_sub(offset);
        if end == 0 {
            return Ok(vec![]);
        }
        let start = end.saturating_sub(max_results).max(1);

        let range = format!("{}:{}", start, end);
        let fetches: Vec<_> = session
            .fetch(
                range,
                "(UID FLAGS ENVELOPE BODY.PEEK[HEADER.FIELDS (DATE FROM SUBJECT)] RFC822.SIZE)",
            )
            .await
            .context("Failed to fetch messages")?
            .collect::<Vec<_>>()
            .await;

        let mut items: Vec<EmailListItem> = Vec::new();

        for fetch_result in &fetches {
            if let Ok(fetch) = fetch_result {
                if let Some(uid) = fetch.uid {
                    let item = self.parse_fetch_to_list_item(uid, folder, fetch);
                    items.push(item);
                }
            }
        }

        items.reverse();
        Ok(items)
    }

    async fn get_message(&self, folder: &str, uid: u32) -> Result<Email> {
        let mut guard = self.get_session().await?;
        let session = guard.as_mut().context("No IMAP session")?;

        session
            .select(folder)
            .await
            .context("Failed to select folder")?;

        let uid_str = uid.to_string();
        let fetches: Vec<_> = session
            .uid_fetch(&uid_str, "(FLAGS BODY[])")
            .await
            .context("Failed to fetch message")?
            .collect::<Vec<_>>()
            .await;

        let fetch = fetches
            .into_iter()
            .next()
            .context("Message not found")?
            .context("Failed to fetch message")?;

        let raw = fetch.body().context("No message body")?;
        let flags: Vec<Flag<'_>> = fetch.flags().collect();

        self.parse_raw_email(uid, folder, raw, &flags)
    }

    async fn send_email(
        &self,
        from: &str,
        to: Vec<String>,
        cc: Vec<String>,
        bcc: Vec<String>,
        subject: &str,
        body_html: &str,
        body_plain: &str,
    ) -> Result<()> {
        let from_mailbox: Mailbox = from.parse().context("Invalid from address")?;

        let mut builder = Message::builder().from(from_mailbox).subject(subject);

        for addr in &to {
            let mbox: Mailbox = addr.parse().context("Invalid to address")?;
            builder = builder.to(mbox);
        }
        for addr in &cc {
            let mbox: Mailbox = addr.parse().context("Invalid cc address")?;
            builder = builder.cc(mbox);
        }
        for addr in &bcc {
            let mbox: Mailbox = addr.parse().context("Invalid bcc address")?;
            builder = builder.bcc(mbox);
        }

        let email = if !body_html.is_empty() && !body_plain.is_empty() {
            builder.multipart(
                MultiPart::alternative()
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(body_plain.to_string()),
                    )
                    .singlepart(
                        SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(body_html.to_string()),
                    ),
            )?
        } else if !body_html.is_empty() {
            builder.singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_HTML)
                    .body(body_html.to_string()),
            )?
        } else {
            builder.singlepart(
                SinglePart::builder()
                    .header(ContentType::TEXT_PLAIN)
                    .body(body_plain.to_string()),
            )?
        };

        let transport = self.build_smtp_transport().await?;
        transport
            .send(email)
            .await
            .context("Failed to send email via SMTP")?;

        Ok(())
    }

    async fn set_flags(
        &self,
        folder: &str,
        uid: u32,
        flags: &[ImapFlag],
        add: bool,
    ) -> Result<()> {
        let mut guard = self.get_session().await?;
        let session = guard.as_mut().context("No IMAP session")?;

        session
            .select(folder)
            .await
            .context("Failed to select folder")?;

        let flag_str = flags
            .iter()
            .map(|f| f.to_imap_str())
            .collect::<Vec<_>>()
            .join(" ");

        let uid_str = uid.to_string();
        if add {
            session
                .uid_store(&uid_str, format!("+FLAGS ({})", flag_str))
                .await
                .context("Failed to add flags")?;
        } else {
            session
                .uid_store(&uid_str, format!("-FLAGS ({})", flag_str))
                .await
                .context("Failed to remove flags")?;
        }

        Ok(())
    }

    async fn move_message(&self, from_folder: &str, uid: u32, to_folder: &str) -> Result<()> {
        let mut guard = self.get_session().await?;
        let session = guard.as_mut().context("No IMAP session")?;

        session
            .select(from_folder)
            .await
            .context("Failed to select source folder")?;

        let uid_str = uid.to_string();

        // Try MOVE extension first (RFC 6851)
        match session.uid_mv(&uid_str, to_folder).await {
            Ok(_) => Ok(()),
            Err(_) => {
                // Fallback: COPY + STORE \Deleted + EXPUNGE
                session
                    .uid_copy(&uid_str, to_folder)
                    .await
                    .context("Failed to copy message")?;
                session
                    .uid_store(&uid_str, "+FLAGS (\\Deleted)")
                    .await
                    .context("Failed to mark as deleted")?;
                session
                    .expunge()
                    .await
                    .context("Failed to expunge")?;
                Ok(())
            }
        }
    }

    async fn delete_message(&self, folder: &str, uid: u32) -> Result<()> {
        let mut guard = self.get_session().await?;
        let session = guard.as_mut().context("No IMAP session")?;

        session
            .select(folder)
            .await
            .context("Failed to select folder")?;

        let uid_str = uid.to_string();
        session
            .uid_store(&uid_str, "+FLAGS (\\Deleted)")
            .await
            .context("Failed to mark as deleted")?;

        session.expunge().await.context("Failed to expunge")?;

        Ok(())
    }

    async fn list_folders(&self) -> Result<Vec<Folder>> {
        let mut guard = self.get_session().await?;
        let session = guard.as_mut().context("No IMAP session")?;

        let names: Vec<_> = session
            .list(Some(""), Some("*"))
            .await
            .context("Failed to list folders")?
            .collect::<Vec<_>>()
            .await;

        let mut folders = Vec::new();
        for name_result in &names {
            let name = match name_result {
                Ok(n) => n,
                Err(_) => continue,
            };
            let full_name = name.name().to_string();
            let display_name = full_name
                .rsplit('/')
                .next()
                .unwrap_or(&full_name)
                .to_string();

            let special = self.detect_special_folder(&full_name, name.attributes());

            folders.push(Folder {
                name: full_name,
                display_name,
                special,
                delimiter: name.delimiter().map(|s| s.to_string()),
            });
        }

        Ok(folders)
    }
}
