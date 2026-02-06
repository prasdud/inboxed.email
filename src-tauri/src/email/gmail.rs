use super::types::*;
use anyhow::{Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::Client;

const GMAIL_API_BASE: &str = "https://gmail.googleapis.com/gmail/v1/users/me";

pub struct GmailClient {
    client: Client,
    access_token: String,
}

impl GmailClient {
    pub fn new(access_token: String) -> Self {
        Self {
            client: Client::new(),
            access_token,
        }
    }

    /// List emails with optional query and max results
    pub async fn list_messages(
        &self,
        max_results: Option<u32>,
        query: Option<&str>,
        page_token: Option<&str>,
    ) -> Result<GmailListResponse> {
        let mut url = format!("{}/messages", GMAIL_API_BASE);
        let mut params = vec![];

        if let Some(max) = max_results {
            params.push(format!("maxResults={}", max));
        }
        if let Some(q) = query {
            params.push(format!("q={}", urlencoding::encode(q)));
        }
        if let Some(token) = page_token {
            params.push(format!("pageToken={}", token));
        }

        if !params.is_empty() {
            url.push_str("?");
            url.push_str(&params.join("&"));
        }

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .context("Failed to fetch messages list")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Gmail API error ({}): {}", status, text));
        }

        response
            .json()
            .await
            .context("Failed to parse messages list response")
    }

    /// Get a single message by ID
    pub async fn get_message(&self, message_id: &str) -> Result<GmailMessage> {
        let url = format!("{}/messages/{}", GMAIL_API_BASE, message_id);

        let response = self
            .client
            .get(&url)
            .bearer_auth(&self.access_token)
            .query(&[("format", "full")])
            .send()
            .await
            .context("Failed to fetch message")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Gmail API error ({}): {}", status, text));
        }

        response
            .json()
            .await
            .context("Failed to parse message response")
    }

    /// Parse a GmailMessage into our Email type
    pub fn parse_email(&self, msg: GmailMessage) -> Email {
        let headers = msg
            .payload
            .as_ref()
            .map(|p| &p.headers)
            .cloned()
            .unwrap_or_default();

        let subject =
            Self::get_header(&headers, "Subject").unwrap_or_else(|| "(No Subject)".to_string());
        let from = Self::get_header(&headers, "From").unwrap_or_else(|| "Unknown".to_string());
        let to_str = Self::get_header(&headers, "To").unwrap_or_default();
        let date = Self::get_header(&headers, "Date").unwrap_or_default();

        // Parse internal_date (Unix timestamp in milliseconds) to seconds
        let date_timestamp = msg
            .internal_date
            .as_ref()
            .and_then(|d| d.parse::<i64>().ok())
            .map(|ms| ms / 1000) // Convert milliseconds to seconds
            .unwrap_or_else(|| chrono::Utc::now().timestamp());

        let from_email = Self::extract_email(&from);
        let to = to_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let is_read = !msg.label_ids.contains(&"UNREAD".to_string());
        let is_starred = msg.label_ids.contains(&"STARRED".to_string());

        let (body_html, body_plain) = if let Some(payload) = &msg.payload {
            Self::extract_body(payload)
        } else {
            (None, None)
        };

        let has_attachments = Self::has_attachments(msg.payload.as_ref());

        Email {
            id: msg.id,
            thread_id: msg.thread_id,
            subject,
            from,
            from_email,
            to,
            date,
            date_timestamp,
            snippet: msg.snippet,
            body_html,
            body_plain,
            labels: msg.label_ids,
            is_read,
            is_starred,
            has_attachments,
        }
    }

    /// Convert Email to EmailListItem
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

    fn get_header(headers: &[MessageHeader], name: &str) -> Option<String> {
        headers
            .iter()
            .find(|h| h.name.eq_ignore_ascii_case(name))
            .map(|h| h.value.clone())
    }

    fn extract_email(from: &str) -> String {
        if let Some(start) = from.rfind('<') {
            if let Some(end) = from.rfind('>') {
                return from[start + 1..end].to_string();
            }
        }
        from.to_string()
    }

    fn extract_body(payload: &MessagePayload) -> (Option<String>, Option<String>) {
        let mut html = None;
        let mut plain = None;

        // Check direct body
        if let Some(body) = &payload.body {
            if let Some(data) = &body.data {
                if let Some(mime) = &payload.mime_type {
                    let decoded = Self::decode_base64(data);
                    if mime.contains("html") {
                        html = decoded;
                    } else {
                        plain = decoded;
                    }
                }
            }
        }

        // Check parts
        if let Some(parts) = &payload.parts {
            for part in parts {
                Self::extract_body_from_part(part, &mut html, &mut plain);
            }
        }

        (html, plain)
    }

    fn extract_body_from_part(
        part: &MessagePart,
        html: &mut Option<String>,
        plain: &mut Option<String>,
    ) {
        if let Some(body) = &part.body {
            if let Some(data) = &body.data {
                if let Some(mime) = &part.mime_type {
                    let decoded = Self::decode_base64(data);
                    if mime.contains("html") && html.is_none() {
                        *html = decoded;
                    } else if mime.contains("plain") && plain.is_none() {
                        *plain = decoded;
                    }
                }
            }
        }

        // Recursively check nested parts
        if let Some(parts) = &part.parts {
            for nested_part in parts {
                Self::extract_body_from_part(nested_part, html, plain);
            }
        }
    }

    fn decode_base64(data: &str) -> Option<String> {
        let data = data.replace('-', "+").replace('_', "/");
        STANDARD
            .decode(data.as_bytes())
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
    }

    fn has_attachments(payload: Option<&MessagePayload>) -> bool {
        if let Some(payload) = payload {
            if let Some(parts) = &payload.parts {
                return Self::check_parts_for_attachments(parts);
            }
        }
        false
    }

    fn check_parts_for_attachments(parts: &[MessagePart]) -> bool {
        for part in parts {
            if let Some(filename) = &part.filename {
                if !filename.is_empty() {
                    return true;
                }
            }
            if let Some(nested_parts) = &part.parts {
                if Self::check_parts_for_attachments(nested_parts) {
                    return true;
                }
            }
        }
        false
    }

    /// Send an email
    pub async fn send_email(
        &self,
        to: Vec<String>,
        subject: String,
        body: String,
        cc: Option<Vec<String>>,
        bcc: Option<Vec<String>>,
    ) -> Result<String> {
        use base64::{engine::general_purpose::URL_SAFE, Engine};

        // Build email in RFC 2822 format
        let mut email_content = String::new();
        email_content.push_str(&format!("To: {}\r\n", to.join(", ")));

        if let Some(cc_addrs) = cc {
            if !cc_addrs.is_empty() {
                email_content.push_str(&format!("Cc: {}\r\n", cc_addrs.join(", ")));
            }
        }

        if let Some(bcc_addrs) = bcc {
            if !bcc_addrs.is_empty() {
                email_content.push_str(&format!("Bcc: {}\r\n", bcc_addrs.join(", ")));
            }
        }

        email_content.push_str(&format!("Subject: {}\r\n", subject));
        email_content.push_str("Content-Type: text/html; charset=utf-8\r\n");
        email_content.push_str("\r\n");
        email_content.push_str(&body);

        // Base64 encode
        let encoded = URL_SAFE.encode(email_content.as_bytes());

        let url = format!("{}/messages/send", GMAIL_API_BASE);
        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.access_token)
            .json(&serde_json::json!({
                "raw": encoded
            }))
            .send()
            .await
            .context("Failed to send email")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Failed to send email ({}): {}",
                status,
                text
            ));
        }

        let result: serde_json::Value = response.json().await?;
        Ok(result["id"].as_str().unwrap_or("").to_string())
    }

    /// Modify email labels (for delete, archive, mark read, star, etc.)
    pub async fn modify_labels(
        &self,
        message_id: &str,
        add_labels: Vec<String>,
        remove_labels: Vec<String>,
    ) -> Result<()> {
        let url = format!("{}/messages/{}/modify", GMAIL_API_BASE, message_id);

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.access_token)
            .json(&serde_json::json!({
                "addLabelIds": add_labels,
                "removeLabelIds": remove_labels
            }))
            .send()
            .await
            .context("Failed to modify labels")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Failed to modify labels ({}): {}",
                status,
                text
            ));
        }

        Ok(())
    }

    /// Move to trash
    pub async fn trash_email(&self, message_id: &str) -> Result<()> {
        let url = format!("{}/messages/{}/trash", GMAIL_API_BASE, message_id);

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .context("Failed to trash email")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Failed to trash email ({}): {}",
                status,
                text
            ));
        }

        Ok(())
    }

    /// Delete permanently
    pub async fn delete_email(&self, message_id: &str) -> Result<()> {
        let url = format!("{}/messages/{}", GMAIL_API_BASE, message_id);

        let response = self
            .client
            .delete(&url)
            .bearer_auth(&self.access_token)
            .send()
            .await
            .context("Failed to delete email")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Failed to delete email ({}): {}",
                status,
                text
            ));
        }

        Ok(())
    }
}
