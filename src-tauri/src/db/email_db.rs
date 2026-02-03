use rusqlite::{Connection, Result, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use anyhow::{Context, Result as AnyhowResult};
use chrono::Utc;

use crate::email::types::Email;
use super::schema::create_tables;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailInsight {
    pub email_id: String,
    pub summary: Option<String>,
    pub priority: String,
    pub priority_score: f64,
    pub category: Option<String>,
    pub insights: Option<String>,
    pub action_items: Option<String>,
    pub has_deadline: bool,
    pub has_meeting: bool,
    pub has_financial: bool,
    pub sentiment: Option<String>,
    pub indexed_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailWithInsight {
    pub id: String,
    pub thread_id: String,
    pub subject: String,
    pub from_name: String,
    pub from_email: String,
    pub to_emails: Vec<String>,
    pub date: i64,
    pub snippet: String,
    pub is_read: bool,
    pub is_starred: bool,
    pub has_attachments: bool,
    pub priority: String,
    pub priority_score: f64,
    pub category: Option<String>,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexingStatus {
    pub is_indexing: bool,
    pub total_emails: i64,
    pub processed_emails: i64,
    pub last_indexed_at: Option<i64>,
    pub error_message: Option<String>,
}

pub struct EmailDatabase {
    conn: Arc<Mutex<Connection>>,
}

impl EmailDatabase {
    pub fn new(db_path: PathBuf) -> AnyhowResult<Self> {
        let conn = Connection::open(db_path)
            .context("Failed to open database")?;

        create_tables(&conn)
            .context("Failed to create database tables")?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    // Store or update an email
    pub fn store_email(&self, email: &Email) -> AnyhowResult<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().timestamp();

        conn.execute(
            "INSERT OR REPLACE INTO emails
            (id, thread_id, subject, from_name, from_email, to_emails, date, snippet,
             body_html, body_plain, is_read, is_starred, has_attachments, labels,
             created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                &email.id,
                &email.thread_id,
                &email.subject,
                &email.from,
                &email.from_email,
                serde_json::to_string(&email.to)?,
                &email.date,
                &email.snippet,
                &email.body_html,
                &email.body_plain,
                email.is_read as i32,
                email.is_starred as i32,
                email.has_attachments as i32,
                serde_json::to_string(&email.labels)?,
                now,
                now,
            ],
        )?;

        Ok(())
    }

    // Store AI insights for an email
    pub fn store_insights(&self, insight: &EmailInsight) -> AnyhowResult<()> {
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT OR REPLACE INTO email_insights
            (email_id, summary, priority, priority_score, category, insights,
             action_items, has_deadline, has_meeting, has_financial, sentiment, indexed_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                &insight.email_id,
                &insight.summary,
                &insight.priority,
                insight.priority_score,
                &insight.category,
                &insight.insights,
                &insight.action_items,
                insight.has_deadline as i32,
                insight.has_meeting as i32,
                insight.has_financial as i32,
                &insight.sentiment,
                insight.indexed_at,
            ],
        )?;

        Ok(())
    }

    // Get emails sorted by priority
    pub fn get_emails_by_priority(&self, limit: i64, offset: i64) -> AnyhowResult<Vec<EmailWithInsight>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT e.id, e.thread_id, e.subject, e.from_name, e.from_email, e.to_emails,
                    e.date, e.snippet, e.is_read, e.is_starred, e.has_attachments,
                    COALESCE(i.priority, 'MEDIUM') as priority,
                    COALESCE(i.priority_score, 0.5) as priority_score,
                    i.category, i.summary
             FROM emails e
             LEFT JOIN email_insights i ON e.id = i.email_id
             ORDER BY COALESCE(i.priority_score, 0.5) DESC, e.date DESC
             LIMIT ?1 OFFSET ?2"
        )?;

        let emails = stmt.query_map(params![limit, offset], |row| {
            Ok(EmailWithInsight {
                id: row.get(0)?,
                thread_id: row.get(1)?,
                subject: row.get(2)?,
                from_name: row.get(3)?,
                from_email: row.get(4)?,
                to_emails: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                date: row.get(6)?,
                snippet: row.get(7)?,
                is_read: row.get::<_, i32>(8)? != 0,
                is_starred: row.get::<_, i32>(9)? != 0,
                has_attachments: row.get::<_, i32>(10)? != 0,
                priority: row.get(11)?,
                priority_score: row.get(12)?,
                category: row.get(13)?,
                summary: row.get(14)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(emails)
    }

    // Get emails by category
    pub fn get_emails_by_category(&self, category: &str, limit: i64) -> AnyhowResult<Vec<EmailWithInsight>> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT e.id, e.thread_id, e.subject, e.from_name, e.from_email, e.to_emails,
                    e.date, e.snippet, e.is_read, e.is_starred, e.has_attachments,
                    i.priority, i.priority_score, i.category, i.summary
             FROM emails e
             INNER JOIN email_insights i ON e.id = i.email_id
             WHERE i.category = ?1
             ORDER BY i.priority_score DESC, e.date DESC
             LIMIT ?2"
        )?;

        let emails = stmt.query_map(params![category, limit], |row| {
            Ok(EmailWithInsight {
                id: row.get(0)?,
                thread_id: row.get(1)?,
                subject: row.get(2)?,
                from_name: row.get(3)?,
                from_email: row.get(4)?,
                to_emails: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                date: row.get(6)?,
                snippet: row.get(7)?,
                is_read: row.get::<_, i32>(8)? != 0,
                is_starred: row.get::<_, i32>(9)? != 0,
                has_attachments: row.get::<_, i32>(10)? != 0,
                priority: row.get(11)?,
                priority_score: row.get(12)?,
                category: row.get(13)?,
                summary: row.get(14)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(emails)
    }

    // Get emails from today
    pub fn get_emails_from_today(&self) -> AnyhowResult<Vec<EmailWithInsight>> {
        let conn = self.conn.lock().unwrap();
        let today_start = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();

        let mut stmt = conn.prepare(
            "SELECT e.id, e.thread_id, e.subject, e.from_name, e.from_email, e.to_emails,
                    e.date, e.snippet, e.is_read, e.is_starred, e.has_attachments,
                    COALESCE(i.priority, 'MEDIUM') as priority,
                    COALESCE(i.priority_score, 0.5) as priority_score,
                    i.category, i.summary
             FROM emails e
             LEFT JOIN email_insights i ON e.id = i.email_id
             WHERE e.date >= ?1
             ORDER BY e.date DESC"
        )?;

        let emails = stmt.query_map(params![today_start], |row| {
            Ok(EmailWithInsight {
                id: row.get(0)?,
                thread_id: row.get(1)?,
                subject: row.get(2)?,
                from_name: row.get(3)?,
                from_email: row.get(4)?,
                to_emails: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                date: row.get(6)?,
                snippet: row.get(7)?,
                is_read: row.get::<_, i32>(8)? != 0,
                is_starred: row.get::<_, i32>(9)? != 0,
                has_attachments: row.get::<_, i32>(10)? != 0,
                priority: row.get(11)?,
                priority_score: row.get(12)?,
                category: row.get(13)?,
                summary: row.get(14)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(emails)
    }

    // Search emails by text
    pub fn search_emails(&self, query: &str, limit: i64) -> AnyhowResult<Vec<EmailWithInsight>> {
        let conn = self.conn.lock().unwrap();
        let search_pattern = format!("%{}%", query);

        let mut stmt = conn.prepare(
            "SELECT e.id, e.thread_id, e.subject, e.from_name, e.from_email, e.to_emails,
                    e.date, e.snippet, e.is_read, e.is_starred, e.has_attachments,
                    COALESCE(i.priority, 'MEDIUM') as priority,
                    COALESCE(i.priority_score, 0.5) as priority_score,
                    i.category, i.summary
             FROM emails e
             LEFT JOIN email_insights i ON e.id = i.email_id
             WHERE e.subject LIKE ?1 OR e.from_name LIKE ?1 OR e.snippet LIKE ?1
                   OR COALESCE(i.summary, '') LIKE ?1
             ORDER BY e.date DESC
             LIMIT ?2"
        )?;

        let emails = stmt.query_map(params![&search_pattern, limit], |row| {
            Ok(EmailWithInsight {
                id: row.get(0)?,
                thread_id: row.get(1)?,
                subject: row.get(2)?,
                from_name: row.get(3)?,
                from_email: row.get(4)?,
                to_emails: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                date: row.get(6)?,
                snippet: row.get(7)?,
                is_read: row.get::<_, i32>(8)? != 0,
                is_starred: row.get::<_, i32>(9)? != 0,
                has_attachments: row.get::<_, i32>(10)? != 0,
                priority: row.get(11)?,
                priority_score: row.get(12)?,
                category: row.get(13)?,
                summary: row.get(14)?,
            })
        })?.collect::<Result<Vec<_>, _>>()?;

        Ok(emails)
    }

    // Update indexing status
    pub fn update_indexing_status(
        &self,
        is_indexing: bool,
        total: Option<i64>,
        processed: Option<i64>,
        error: Option<String>,
    ) -> AnyhowResult<()> {
        let conn = self.conn.lock().unwrap();

        if let Some(total) = total {
            conn.execute(
                "UPDATE indexing_status SET total_emails = ?1 WHERE id = 1",
                params![total],
            )?;
        }

        if let Some(processed) = processed {
            conn.execute(
                "UPDATE indexing_status SET processed_emails = ?1 WHERE id = 1",
                params![processed],
            )?;
        }

        conn.execute(
            "UPDATE indexing_status SET is_indexing = ?1 WHERE id = 1",
            params![is_indexing as i32],
        )?;

        if !is_indexing {
            let now = Utc::now().timestamp();
            conn.execute(
                "UPDATE indexing_status SET last_indexed_at = ?1 WHERE id = 1",
                params![now],
            )?;
        }

        if let Some(error_msg) = error {
            conn.execute(
                "UPDATE indexing_status SET error_message = ?1 WHERE id = 1",
                params![error_msg],
            )?;
        }

        Ok(())
    }

    // Get indexing status
    pub fn get_indexing_status(&self) -> AnyhowResult<IndexingStatus> {
        let conn = self.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT is_indexing, total_emails, processed_emails, last_indexed_at, error_message
             FROM indexing_status WHERE id = 1"
        )?;

        let status = stmt.query_row([], |row| {
            Ok(IndexingStatus {
                is_indexing: row.get::<_, i32>(0)? != 0,
                total_emails: row.get(1)?,
                processed_emails: row.get(2)?,
                last_indexed_at: row.get(3)?,
                error_message: row.get(4)?,
            })
        })?;

        Ok(status)
    }

    // Get total count of emails
    pub fn get_email_count(&self) -> AnyhowResult<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM emails", [], |row| row.get(0))?;
        Ok(count)
    }

    // Get count of indexed emails
    pub fn get_indexed_count(&self) -> AnyhowResult<i64> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM email_insights", [], |row| row.get(0))?;
        Ok(count)
    }
}
