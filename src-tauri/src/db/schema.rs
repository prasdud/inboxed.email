use rusqlite::{Connection, Result};

pub fn create_tables(conn: &Connection) -> Result<()> {
    // Check if we need to migrate the date column from TEXT to INTEGER
    migrate_date_column_if_needed(conn)?;
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

    // Email embeddings table - stores vector embeddings for RAG
    conn.execute(
        "CREATE TABLE IF NOT EXISTS email_embeddings (
            email_id TEXT PRIMARY KEY,
            embedding BLOB NOT NULL,
            embedding_model TEXT NOT NULL,
            text_hash TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (email_id) REFERENCES emails(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // Embedding status table - track embedding progress
    conn.execute(
        "CREATE TABLE IF NOT EXISTS embedding_status (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            is_embedding INTEGER NOT NULL DEFAULT 0,
            total_emails INTEGER NOT NULL DEFAULT 0,
            embedded_emails INTEGER NOT NULL DEFAULT 0,
            current_model TEXT,
            last_embedded_at INTEGER,
            error_message TEXT
        )",
        [],
    )?;

    // Initialize indexing status if not exists
    conn.execute("INSERT OR IGNORE INTO indexing_status (id) VALUES (1)", [])?;

    // Initialize embedding status if not exists
    conn.execute("INSERT OR IGNORE INTO embedding_status (id) VALUES (1)", [])?;

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

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_embeddings_model ON email_embeddings(embedding_model)",
        [],
    )?;

    Ok(())
}

/// Migrates the date column from TEXT to INTEGER if needed
/// This handles databases created with the old schema
fn migrate_date_column_if_needed(conn: &Connection) -> Result<()> {
    // Check if emails table exists and has TEXT date column
    let table_exists: bool = conn
        .query_row(
            "SELECT count(*) > 0 FROM sqlite_master WHERE type='table' AND name='emails'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(false);

    if !table_exists {
        return Ok(()); // No migration needed, table will be created fresh
    }

    // Check the column type using PRAGMA table_info
    let date_is_text: bool = conn
        .query_row(
            "SELECT type FROM pragma_table_info('emails') WHERE name = 'date'",
            [],
            |row| {
                let col_type: String = row.get(0)?;
                Ok(col_type.to_uppercase() == "TEXT")
            },
        )
        .unwrap_or(false);

    if !date_is_text {
        return Ok(()); // No migration needed, column is already INTEGER
    }

    eprintln!("Migrating emails table: converting date column from TEXT to INTEGER...");

    // Begin transaction for the migration
    conn.execute("BEGIN TRANSACTION", [])?;

    // Create new table with correct schema
    conn.execute(
        "CREATE TABLE IF NOT EXISTS emails_new (
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

    // Copy data, parsing date strings to timestamps
    // Try to handle various date formats
    conn.execute(
        "INSERT INTO emails_new 
         SELECT id, thread_id, subject, from_name, from_email, to_emails,
                CASE 
                    WHEN typeof(date) = 'integer' THEN date
                    WHEN date GLOB '[0-9]*' THEN CAST(date AS INTEGER)
                    ELSE strftime('%s', date)
                END as date,
                snippet, body_html, body_plain, is_read, is_starred, 
                has_attachments, labels, created_at, updated_at
         FROM emails WHERE date IS NOT NULL",
        [],
    )?;

    // Drop old table
    conn.execute("DROP TABLE emails", [])?;

    // Rename new table
    conn.execute("ALTER TABLE emails_new RENAME TO emails", [])?;

    // Recreate indexes
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_emails_date ON emails(date DESC)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_emails_thread ON emails(thread_id)",
        [],
    )?;

    conn.execute("COMMIT", [])?;

    eprintln!("Migration complete: date column converted to INTEGER");

    Ok(())
}
