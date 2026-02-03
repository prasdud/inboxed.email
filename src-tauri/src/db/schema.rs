use rusqlite::{Connection, Result};

pub fn create_tables(conn: &Connection) -> Result<()> {
    // Emails table - stores email metadata and content
    conn.execute(
        "CREATE TABLE IF NOT EXISTS emails (
            id TEXT PRIMARY KEY,
            thread_id TEXT NOT NULL,
            subject TEXT NOT NULL,
            from_name TEXT NOT NULL,
            from_email TEXT NOT NULL,
            to_emails TEXT NOT NULL,
            date INTEGER NOT NULL,
            snippet TEXT NOT NULL,
            body_html TEXT,
            body_plain TEXT,
            is_read INTEGER NOT NULL DEFAULT 0,
            is_starred INTEGER NOT NULL DEFAULT 0,
            has_attachments INTEGER NOT NULL DEFAULT 0,
            labels TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )",
        [],
    )?;

    // AI Insights table - stores AI-generated data
    conn.execute(
        "CREATE TABLE IF NOT EXISTS email_insights (
            email_id TEXT PRIMARY KEY,
            summary TEXT,
            priority TEXT NOT NULL DEFAULT 'MEDIUM',
            priority_score REAL NOT NULL DEFAULT 0.5,
            category TEXT,
            insights TEXT,
            action_items TEXT,
            has_deadline INTEGER NOT NULL DEFAULT 0,
            has_meeting INTEGER NOT NULL DEFAULT 0,
            has_financial INTEGER NOT NULL DEFAULT 0,
            sentiment TEXT,
            indexed_at INTEGER NOT NULL,
            FOREIGN KEY (email_id) REFERENCES emails(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Indexing status table - track email processing
    conn.execute(
        "CREATE TABLE IF NOT EXISTS indexing_status (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            is_indexing INTEGER NOT NULL DEFAULT 0,
            total_emails INTEGER NOT NULL DEFAULT 0,
            processed_emails INTEGER NOT NULL DEFAULT 0,
            last_indexed_at INTEGER,
            error_message TEXT
        )",
        [],
    )?;

    // Initialize indexing status if not exists
    conn.execute(
        "INSERT OR IGNORE INTO indexing_status (id) VALUES (1)",
        [],
    )?;

    // Create indexes for performance
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_emails_date ON emails(date DESC)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_emails_thread ON emails(thread_id)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_insights_priority ON email_insights(priority_score DESC)",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_insights_category ON email_insights(category)",
        [],
    )?;

    Ok(())
}
